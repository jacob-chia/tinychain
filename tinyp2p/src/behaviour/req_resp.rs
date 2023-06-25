use std::{io, iter, time::Duration};

use async_trait::async_trait;
use libp2p::{
    futures::prelude::*,
    request_response::{self, Behaviour, Codec, ProtocolName, ProtocolSupport},
};

pub type ResponseType = Result<Vec<u8>, ()>;

/// The behaviour builder.
#[derive(Debug, Clone)]
pub struct BehaviourBuilder {
    /// The keep-alive timeout of idle connections.
    connection_keep_alive: Duration,
    /// The timeout for inbound and outbound requests.
    request_timeout: Duration,
    /// The maximum size of requests.
    max_request_size: usize,
    /// The maximum size of responses.
    max_response_size: usize,
}

impl Default for BehaviourBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviourBuilder {
    /// Create a new `BehaviourBuilder`.
    pub fn new() -> Self {
        Self {
            connection_keep_alive: Duration::from_secs(10),
            request_timeout: Duration::from_secs(10),
            max_request_size: usize::MAX,
            max_response_size: usize::MAX,
        }
    }

    /// Set the keep-alive timeout of idle connections.
    pub fn with_connection_keep_alive(mut self, connection_keep_alive: Option<u64>) -> Self {
        if let Some(secs) = connection_keep_alive {
            self.connection_keep_alive = Duration::from_secs(secs);
        }
        self
    }

    /// Set the timeout for inbound and outbound requests.
    pub fn with_request_timeout(mut self, request_timeout: Option<u64>) -> Self {
        if let Some(secs) = request_timeout {
            self.request_timeout = Duration::from_secs(secs);
        }
        self
    }

    /// Set the maximum size of requests.
    pub fn with_max_request_size(mut self, max_request_size: Option<usize>) -> Self {
        if let Some(max_request_size) = max_request_size {
            self.max_request_size = max_request_size;
        }
        self
    }

    /// Set the maximum size of responses.
    pub fn with_max_response_size(mut self, max_response_size: Option<usize>) -> Self {
        if let Some(max_response_size) = max_response_size {
            self.max_response_size = max_response_size;
        }
        self
    }

    /// Build a `Behaviour` with the given configuration.
    pub fn build(self) -> Behaviour<GenericCodec> {
        let codec = GenericCodec {
            max_request_size: self.max_request_size,
            max_response_size: self.max_response_size,
        };

        let protocols = iter::once((GenericProtocol, ProtocolSupport::Full));

        let mut cfg = request_response::Config::default();
        cfg.set_connection_keep_alive(self.connection_keep_alive);
        cfg.set_request_timeout(self.request_timeout);

        Behaviour::new(codec, protocols, cfg)
    }
}

#[derive(Debug, Clone)]
pub struct GenericProtocol;

impl ProtocolName for GenericProtocol {
    fn protocol_name(&self) -> &[u8] {
        b"/tinyp2p/req-resp/1.0.0"
    }
}

/// Generic request-response codec.
/// The format of the request and response is a length-prefixed payload.
/// The length is encoded as a varint (variable-width integer).
/// [What is a varint?](https://developers.google.com/protocol-buffers/docs/encoding#varints)
#[derive(Debug, Clone)]
pub struct GenericCodec {
    /// Maximum size of requests.
    max_request_size: usize,
    /// Maximum size of responses.
    max_response_size: usize,
}

#[async_trait]
impl Codec for GenericCodec {
    type Protocol = GenericProtocol;
    type Request = Vec<u8>;
    type Response = ResponseType;

    async fn read_request<T>(
        &mut self,
        _: &Self::Protocol,
        mut io: &mut T,
    ) -> io::Result<Self::Request>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Read the length.
        let length = unsigned_varint::aio::read_usize(&mut io)
            .await
            .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?;

        if length > self.max_request_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Request size exceeds limit: {} > {}",
                    length, self.max_request_size
                ),
            ));
        }

        // Read the payload.
        let mut buffer = vec![0; length];
        io.read_exact(&mut buffer).await?;
        Ok(buffer)
    }

    async fn read_response<T>(
        &mut self,
        _: &Self::Protocol,
        mut io: &mut T,
    ) -> io::Result<Self::Response>
    where
        T: AsyncRead + Unpin + Send,
    {
        // Note that this function returns a `Result<Result<...>>`.
        // - Returning `Err` is considered as a protocol error.
        // - Returning `Ok(Err())` indicates that the response has been successfully read,
        //   and the content is an error.

        // Read the length.
        let length = match unsigned_varint::aio::read_usize(&mut io).await {
            Ok(l) => l,
            Err(unsigned_varint::io::ReadError::Io(err))
                if matches!(err.kind(), io::ErrorKind::UnexpectedEof) =>
            {
                return Ok(Err(()))
            }
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidInput, err)),
        };

        if length > self.max_request_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Response size exceeds limit: {} > {}",
                    length, self.max_response_size
                ),
            ));
        }

        // Read the payload.
        let mut buffer = vec![0; length];
        io.read_exact(&mut buffer).await?;
        Ok(Ok(buffer))
    }

    async fn write_request<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        req: Self::Request,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // Check the length.
        if req.len() > self.max_request_size {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "Request size exceeds limit: {} > {}",
                    req.len(),
                    self.max_request_size
                ),
            ));
        }

        // Write the length.
        {
            let mut length = unsigned_varint::encode::usize_buffer();
            io.write_all(unsigned_varint::encode::usize(req.len(), &mut length))
                .await?;
        }

        // Write the payload.
        io.write_all(&req).await?;

        io.close().await?;
        Ok(())
    }

    async fn write_response<T>(
        &mut self,
        _: &Self::Protocol,
        io: &mut T,
        res: Self::Response,
    ) -> io::Result<()>
    where
        T: AsyncWrite + Unpin + Send,
    {
        // If `res` is an `Err`, we jump to closing the substream without writing anything on it.
        // The read side will get an `io::ErrorKind::UnexpectedEof` when trying to read the length.
        if let Ok(res) = res {
            // Check the length.
            if res.len() > self.max_request_size {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "Response size exceeds limit: {} > {}",
                        res.len(),
                        self.max_request_size
                    ),
                ));
            }

            // Write the length.
            {
                let mut length = unsigned_varint::encode::usize_buffer();
                io.write_all(unsigned_varint::encode::usize(res.len(), &mut length))
                    .await?;
            }

            // Write the payload.
            io.write_all(&res).await?;
        }

        io.close().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_behaviour_builder() {
        let builder = BehaviourBuilder::default();
        assert_eq!(builder.connection_keep_alive, Duration::from_secs(10));
        assert_eq!(builder.request_timeout, Duration::from_secs(10));
        assert_eq!(builder.max_request_size, usize::MAX);
        assert_eq!(builder.max_response_size, usize::MAX);
    }

    #[test]
    fn custom_behaviour_builder() {
        let builder = BehaviourBuilder::new()
            .with_connection_keep_alive(Some(20))
            .with_request_timeout(Some(20))
            .with_max_request_size(Some(100))
            .with_max_response_size(Some(100));

        assert_eq!(builder.connection_keep_alive, Duration::from_secs(20));
        assert_eq!(builder.request_timeout, Duration::from_secs(20));
        assert_eq!(builder.max_request_size, 100);
        assert_eq!(builder.max_response_size, 100);
    }

    #[tokio::test]
    async fn generic_codec_read_write_request() {
        let mut codec = GenericCodec {
            max_request_size: 10,
            max_response_size: 10,
        };
        let protocol = GenericProtocol;
        let mut buffer = Vec::new();

        // Write request.
        let req = vec![1, 2, 3, 4, 5];
        codec
            .write_request(&protocol, &mut buffer, req.clone())
            .await
            .unwrap();

        // Read request.
        let req2 = codec
            .read_request(&protocol, &mut buffer.as_slice())
            .await
            .unwrap();

        assert_eq!(req, req2);
    }

    #[tokio::test]
    async fn generic_codec_request_too_big() {
        let mut codec = GenericCodec {
            max_request_size: 5,
            max_response_size: 5,
        };
        let protocol = GenericProtocol;
        let mut buffer = Vec::new();

        // Write request.
        let req = vec![1, 2, 3, 4, 5, 6];
        let res = codec
            .write_request(&protocol, &mut buffer, req.clone())
            .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn generic_code_read_write_response() {
        let mut codec = GenericCodec {
            max_request_size: 10,
            max_response_size: 10,
        };
        let protocol = GenericProtocol;
        let mut buffer = Vec::new();

        // Write response.
        let res = vec![1, 2, 3, 4, 5];
        codec
            .write_response(&protocol, &mut buffer, Ok(res.clone()))
            .await
            .unwrap();

        // Read response.
        let res2 = codec
            .read_response(&protocol, &mut buffer.as_slice())
            .await
            .unwrap();

        assert_eq!(res, res2.unwrap());
    }

    #[tokio::test]
    async fn generic_codec_response_too_big() {
        let mut codec = GenericCodec {
            max_request_size: 5,
            max_response_size: 5,
        };
        let protocol = GenericProtocol;
        let mut buffer = Vec::new();

        // Write response.
        let res = vec![1, 2, 3, 4, 5, 6];
        let res = codec
            .write_response(&protocol, &mut buffer, Ok(res.clone()))
            .await;

        assert!(res.is_err());
    }

    #[tokio::test]
    async fn generic_codec_response_is_error() {
        let mut codec = GenericCodec {
            max_request_size: 10,
            max_response_size: 10,
        };
        let protocol = GenericProtocol;
        let mut buffer = Vec::new();

        // Write response.
        codec
            .write_response(&protocol, &mut buffer, Err(()))
            .await
            .unwrap();

        // Read response.
        let res = codec
            .read_response(&protocol, &mut buffer.as_slice())
            .await
            .unwrap();

        assert!(res.is_err());
    }
}
