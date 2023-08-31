- [08 | Network Layer](#08--network-layer)
  - [1 P2P](#1-p2p)
    - [1.1 Implementing the `trait PeerClient`](#11-implementing-the-trait-peerclient)
    - [1.2 Implementing the `trait EventHandler`](#12-implementing-the-trait-eventhandler)
  - [2 HTTP](#2-http)
  - [3 Interfaces That Biz Needs to Provide](#3-interfaces-that-biz-needs-to-provide)
  - [4 Summary](#4-summary)

# 08 | Network Layer

> This is a hands-on tutorial, so please switch to the corresponding code branch before reading.
>
> - Repo: `https://github.com/jacob-chia/tinychain.git`
> - Branch：`git fetch && git switch 08-network`
>
> Important crates used in this lesson:
>
> - [axum](https://docs.rs/axum/latest/axum/): a web application framework that focuses on ergonomics and modularity.

The `network` layer is responsible for interacting with the outside world, including processing HTTP requests and interacting with other peers. In this lesson, we will implement the `network` layer, and in the process, we will identify the interfaces that the `biz` layer needs to provide.

## 1 P2P

The `network::p2p` is the bridge between `biz` and `tinyp2p`, and we need to do two things:

- Wrap `tinyp2p::Client` to implement the `trait PeerClient` defined by `biz`;
- Wrap `biz::Node` to implement the `trait EventHandler` defined by `tinyp2p`;

### 1.1 Implementing the `trait PeerClient`

> See [src/network/p2p.rs](../../src/network/p2p.rs) for details.

```rs
// src/network/p2p.rs

#[derive(Debug, Clone)]
pub struct P2pClient(Client);

// Implement `Deref` so that we can call `Client` methods on `P2pClient`.
impl Deref for P2pClient {
    type Target = Client;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PeerClient for P2pClient {
    fn known_peers(&self) -> Vec<String> {
        self.get_known_peers();
    }

    fn get_block_height(&self, peer_id: &str) -> Result<u64, Error> {
        let req = Request::new_block_height_req();
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        Ok(BlockHeightResp::from(resp).block_height)
    }

    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error> {
        let req = Request::new_blocks_req(from_number);
        let resp: Response = self.blocking_request(peer_id, req.into())?.try_into()?;
        let blocks = BlocksResp::from(resp).blocks;

        Ok(blocks)
    }

    fn broadcast_tx(&self, tx: SignedTx) {
        self.broadcast(Topic::Tx, tx.into());
    }

    fn broadcast_block(&self, block: Block) {
        self.broadcast(Topic::Block, Vec::from(&block));
    }
}
```

### 1.2 Implementing the `trait EventHandler`

> See [src/network/p2p.rs](../../src/network/p2p.rs) for details.

```rs
// src/network/p2p.rs

#[derive(Debug, Clone)]
pub struct EventHandlerImpl<S: State>(Node<S>);

impl<S: State> EventHandler for EventHandlerImpl<S> {
    fn handle_inbound_request(&self, request: Vec<u8>) -> Result<Vec<u8>, P2pError> {
        let req = Request::try_from(request).unwrap();
        let resp = match req.method() {
            Method::Height => {
                let block_height = self.block_height();
                Response::new_block_height_resp(block_height)
            }
            Method::Blocks => {
                let blocks = match req.body.unwrap() {
                    request::Body::BlocksReq(req) => self.get_blocks(req.from_number),
                    _ => vec![],
                };
                Response::new_blocks_resp(blocks)
            }
        };

        Ok(resp.into())
    }

    fn handle_broadcast(&self, topic: &str, message: Vec<u8>) {
        match Topic::from(topic) {
            Topic::Block => {
                if let Ok(block) = Block::try_from(message) {
                    self.handle_broadcast_block(block);
                }
            }
            Topic::Tx => {
                if let Ok(tx) = SignedTx::try_from(message) {
                    self.handle_broadcast_tx(tx);
                }
            }
        }
    }
}
```

As you can see, `EventHandlerImpl` is a wrapper for `biz::Node`, and the `self.xxx()` called internally is interfaces that `biz::Node` needs to provide. As follows:

- `self.block_height()`
- `self.get_blocks(req.from_number)`
- `self.handle_broadcast_block(block)`
- `self.handle_broadcast_tx(tx)`

## 2 HTTP

The implementation of the `network::http` is very simple, it is the forwarding of the `biz` layer interface, just go to the source code [src/network/http/mod.rs](../../src/network/http/mod.rs) for details. But there is a problem: How to return different HTTP Status Code according to the error type?

Simply put, we need to implement the `trait IntoResponse` for the `Error` type, but our global `Error` should not depend on `axum`, so we add a `HTTPError` inside this module, and implement the `IntoResponse` for `HTTPError`.

```rs
// src/network/http/mod.rs

#[derive(thiserror::Error, Debug)]
enum HttpError {
    #[error("Bad request: {0}")]
    BadRequest(Error),
    #[error("Internal server error: {0}")]
    InternalServerError(Error),
}

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        match err {
            Error::BadRequest(..) => HttpError::BadRequest(err),
            _ => HttpError::InternalServerError(err),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self {
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
```

## 3 Interfaces That Biz Needs to Provide

As you can see in the [src/network/http/mod.rs](../../src/network/http/mod.rs) and we mentioned in [1.2 Implementing the `trait EventHandler`](#12-implementing-the-trait-eventhandler), we have identified the interfaces that `biz::Node` needs to provide, as follows:

- `node.block_height()`
- `node.get_blocks(req.from_number)`
- `node.handle_broadcast_block(block)`
- `node.handle_broadcast_tx(tx)`
- `node.get_blocks(params.from_number)`
- `node.get_block(number)`
- `node.last_block_hash()`
- `node.get_balances()`
- `node.next_account_nonce(&params.account)`
- `node.transfer(&tx.from, &tx.to, tx.value, tx.nonce)`

## 4 Summary

After writing the network layer, we once again realized the role of it: do type conversion and interface forwarding, decoupling the business logic from the external environment, and improving the stability of the application.

Furthermore, we also identified the interfaces of `biz::Node`, laying the foundation for implementing the `biz` layer in the next lesson.

---

| [< 07-Tinyp2p: A CSP Concurrency Model](./07-tinyp2p.md) | [09-Biz Layer: How to Do Read/Write Separation? >](./09-biz.md) |
| -------------------------------------------------------- | --------------------------------------------------------------- |
