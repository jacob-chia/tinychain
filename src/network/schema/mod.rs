//! DTOs (Data Transfer Objects) for p2p communication.
//!
//! The DTOs are defined in the `api.v1.proto` file and compiled with the `prost` crate.
//! There are also some helper functions for converting between the DTOs and the internal types.

use prost::Message;

mod v1;
pub use v1::*;

use crate::{error::Error, node};

impl Request {
    pub fn new_best_number_req() -> Self {
        Self {
            method: Method::BestNumber as i32,
            body: Some(request::Body::BestNumberReq(BestNumberReq {})),
        }
    }

    pub fn new_blocks_req(from_number: u64) -> Self {
        Self {
            method: Method::Blocks as i32,
            body: Some(request::Body::BlocksReq(BlocksReq { from_number })),
        }
    }
}

impl Response {
    pub fn new_best_number_resp(best_number: Option<u64>) -> Self {
        Self {
            method: Method::BestNumber as i32,
            body: Some(response::Body::BestNumberResp(BestNumberResp {
                best_number,
            })),
        }
    }

    pub fn new_blocks_resp(blocks: Vec<node::Block>) -> Self {
        Self {
            method: Method::Blocks as i32,
            body: Some(response::Body::BlocksResp(BlocksResp {
                blocks: blocks.into_iter().map(Into::into).collect(),
            })),
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

impl From<Response> for BestNumberResp {
    fn from(value: Response) -> Self {
        match value.body.unwrap() {
            response::Body::BestNumberResp(resp) => resp,
            _ => BestNumberResp { best_number: None },
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

impl TryFrom<Vec<u8>> for Block {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<Block> for Vec<u8> {
    fn from(value: Block) -> Self {
        value.encode_to_vec()
    }
}

impl From<node::Block> for Block {
    fn from(value: node::Block) -> Self {
        Self {
            header: Some(value.header.into()),
            txs: value.txs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Block> for node::Block {
    fn from(value: Block) -> Self {
        Self {
            header: value.header.unwrap().into(),
            txs: value.txs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<node::BlockHeader> for BlockHeader {
    fn from(value: node::BlockHeader) -> Self {
        Self {
            parent_hash: value.parent_hash.into(),
            number: value.number,
            nonce: value.nonce,
            timestamp: value.timestamp,
            author: value.author,
        }
    }
}

impl From<BlockHeader> for node::BlockHeader {
    fn from(value: BlockHeader) -> Self {
        Self {
            parent_hash: value.parent_hash.into(),
            number: value.number,
            nonce: value.nonce,
            timestamp: value.timestamp,
            author: value.author,
        }
    }
}

impl TryFrom<Vec<u8>> for SignedTx {
    type Error = Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        Ok(Self::decode(value.as_slice())?)
    }
}

impl From<SignedTx> for Vec<u8> {
    fn from(value: SignedTx) -> Self {
        value.encode_to_vec()
    }
}

impl From<node::SignedTx> for SignedTx {
    fn from(value: node::SignedTx) -> Self {
        Self {
            tx: Some(value.tx.into()),
            sig: value.sig,
        }
    }
}

impl From<SignedTx> for node::SignedTx {
    fn from(value: SignedTx) -> Self {
        Self {
            tx: value.tx.unwrap().into(),
            sig: value.sig,
        }
    }
}

impl From<node::Tx> for Tx {
    fn from(value: node::Tx) -> Self {
        Self {
            from: value.from,
            to: value.to,
            value: value.value,
            nonce: value.nonce,
            gas: value.gas,
            gas_price: value.gas_price,
            timestamp: value.timestamp,
        }
    }
}

impl From<Tx> for node::Tx {
    fn from(value: Tx) -> Self {
        Self {
            from: value.from,
            to: value.to,
            value: value.value,
            nonce: value.nonce,
            gas: value.gas,
            gas_price: value.gas_price,
            timestamp: value.timestamp,
        }
    }
}
