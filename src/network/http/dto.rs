//! DTO (Data Transfer Object) for HTTP requests and responses.
//!
//! We use JSON as the DTO format for better readability.

use serde::{Deserialize, Serialize};

use crate::{
    schema,
    types::{Hash, Signature},
};

#[derive(Debug, Serialize)]
pub struct Tx {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
    pub gas: u64,
    pub gas_price: u64,
    pub timestamp: u64,
}

#[derive(Debug, Serialize)]
pub struct SignedTx {
    pub tx: Tx,
    pub sig: Signature,
}

#[derive(Debug, Serialize)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub number: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub author: String,
}

#[derive(Debug, Serialize)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<SignedTx>,
}

#[derive(Debug, Serialize)]
pub struct BlockResp {
    pub hash: Hash,
    pub block: Block,
}

#[derive(Debug, Deserialize)]
pub struct GetBlocksReq {
    pub from_number: u64,
}

#[derive(Debug, Deserialize)]
pub struct NonceReq {
    pub account: String,
}

#[derive(Debug, Deserialize)]
pub struct TxReq {
    pub from: String,
    pub to: String,
    pub value: u64,
    pub nonce: u64,
}

impl From<schema::Tx> for Tx {
    fn from(tx: schema::Tx) -> Self {
        Self {
            from: tx.from,
            to: tx.to,
            value: tx.value,
            nonce: tx.nonce,
            gas: tx.gas,
            gas_price: tx.gas_price,
            timestamp: tx.timestamp,
        }
    }
}

impl From<schema::SignedTx> for SignedTx {
    fn from(tx: schema::SignedTx) -> Self {
        Self {
            tx: tx.tx.unwrap().into(),
            sig: Signature::from(tx.sig),
        }
    }
}

impl From<schema::BlockHeader> for BlockHeader {
    fn from(header: schema::BlockHeader) -> Self {
        Self {
            parent_hash: Hash::from(header.parent_hash),
            number: header.number,
            nonce: header.nonce,
            timestamp: header.timestamp,
            author: header.author,
        }
    }
}

impl From<schema::Block> for BlockResp {
    fn from(block: schema::Block) -> Self {
        let hash = block.hash();
        let block = Block {
            header: block.header.unwrap().into(),
            txs: block.txs.into_iter().map(|tx| tx.into()).collect(),
        };

        Self { hash, block }
    }
}
