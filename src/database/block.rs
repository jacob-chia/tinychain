use ethers_core::{types::H256, utils::hash_message};
use serde::{Deserialize, Serialize};
use std::mem;

use super::SignedTx;

const BLOCK_REWORD: u64 = 100;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BlockHeader {
    pub(super) parent: H256,
    pub(super) number: u64,
    pub(super) nonce: u64,
    pub(super) time: u64,
    pub(super) miner: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Block {
    pub(super) header: BlockHeader,
    pub(super) txs: Vec<SignedTx>,
}

impl<'a> Block {
    pub fn builder() -> BlockBuilder<'a> {
        BlockBuilder::default()
    }

    pub fn update_nonce(mut self, nonce: u64) -> Self {
        self.header.nonce = nonce;
        self
    }

    pub fn hash(&self) -> H256 {
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
    parent: H256,
    number: u64,
    nonce: u64,
    time: u64,
    miner: &'a str,
    txs: Vec<SignedTx>,
}

impl<'a> BlockBuilder<'a> {
    pub fn parent(mut self, parent: H256) -> Self {
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
    pub(super) key: H256,
    pub(super) value: Block,
}

impl BlockKV {
    pub fn take_hash(&mut self) -> H256 {
        mem::take(&mut self.key)
    }

    pub fn take_block(&mut self) -> Block {
        mem::take(&mut self.value)
    }
}
