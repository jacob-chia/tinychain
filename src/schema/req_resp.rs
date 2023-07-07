use prost::Message;

use crate::error::Error;

use super::*;

impl Request {
    /// Build a new request to get the block height.
    pub fn new_block_height_req() -> Self {
        Self {
            method: Method::Height as i32,
            body: Some(request::Body::BlockHeightReq(BlockHeightReq {})),
        }
    }

    /// Build a new request to get blocks from the given number.
    pub fn new_blocks_req(from_number: u64) -> Self {
        Self {
            method: Method::Blocks as i32,
            body: Some(request::Body::BlocksReq(BlocksReq { from_number })),
        }
    }
}

impl Response {
    /// Build a new response to get the block height.
    pub fn new_block_height_resp(block_height: u64) -> Self {
        Self {
            method: Method::Height as i32,
            body: Some(response::Body::BlockHeightResp(BlockHeightResp {
                block_height,
            })),
        }
    }

    /// Build a new response to get blocks.
    pub fn new_blocks_resp(blocks: Vec<Block>) -> Self {
        Self {
            method: Method::Blocks as i32,
            body: Some(response::Body::BlocksResp(BlocksResp { blocks })),
        }
    }
}

impl TryFrom<Vec<u8>> for Request {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<Request> for Vec<u8> {
    fn from(value: Request) -> Self {
        value.encode_to_vec()
    }
}

impl TryFrom<Vec<u8>> for Response {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<Response> for Vec<u8> {
    fn from(value: Response) -> Self {
        value.encode_to_vec()
    }
}

impl From<Response> for BlockHeightResp {
    fn from(value: Response) -> Self {
        match value.body.unwrap() {
            response::Body::BlockHeightResp(resp) => resp,
            _ => BlockHeightResp { block_height: 0 },
        }
    }
}

impl From<Response> for BlocksResp {
    fn from(value: Response) -> Self {
        match value.body.unwrap() {
            response::Body::BlocksResp(resp) => resp,
            _ => BlocksResp { blocks: vec![] },
        }
    }
}
