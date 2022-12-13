use ethers_core::utils::hash_message;
use serde::{Deserialize, Serialize};
use std::mem;

use super::SignedTx;
use crate::types::Hash;

const BLOCK_REWORD: u64 = 100;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BlockHeader {
    pub(crate) parent: Hash,
    pub(crate) number: u64,
    pub(crate) nonce: u64,
    pub(crate) time: u64,
    pub(crate) miner: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Block {
    pub(crate) header: BlockHeader,
    pub(crate) txs: Vec<SignedTx>,
}

impl<'a> Block {
    pub fn builder() -> BlockBuilder<'a> {
        BlockBuilder::default()
    }

    pub fn update_nonce(&mut self, nonce: u64) {
        self.header.nonce = nonce;
    }

    pub fn hash(&self) -> Hash {
        let encoded = serde_json::to_string(self).unwrap();
        hash_message(encoded)
    }

    pub fn block_reward(&self) -> u64 {
        let gas_reward: u64 = self.txs.iter().map(|tx| tx.gas_cost()).sum();
        gas_reward + BLOCK_REWORD
    }
}

#[derive(Debug, Default)]
pub struct BlockBuilder<'a> {
    parent: Hash,
    number: u64,
    nonce: u64,
    time: u64,
    miner: &'a str,
    txs: Vec<SignedTx>,
}

impl<'a> BlockBuilder<'a> {
    pub fn parent(mut self, parent: Hash) -> Self {
        self.parent = parent;
        self
    }

    pub fn number(mut self, number: u64) -> Self {
        self.number = number;
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn time(mut self, time: u64) -> Self {
        self.time = time;
        self
    }

    pub fn miner(mut self, miner: &'a str) -> Self {
        self.miner = miner;
        self
    }

    pub fn txs(mut self, txs: Vec<SignedTx>) -> Self {
        self.txs = txs;
        self
    }

    pub fn build(self) -> Block {
        Block {
            header: BlockHeader {
                number: self.number,
                parent: self.parent,
                nonce: self.nonce,
                time: self.time,
                miner: self.miner.to_owned(),
            },
            txs: self.txs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockKV {
    pub(crate) key: Hash,
    pub(crate) value: Block,
}

impl BlockKV {
    pub fn take_block(&mut self) -> Block {
        mem::take(&mut self.value)
    }
}
