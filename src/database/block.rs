use ethers_core::{types::H256, utils::hash_message};
use serde::{Deserialize, Serialize};
use std::mem;

use super::SignedTx;

const BLOCK_REWORD: u64 = 100;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Block {
    header: BlockHeader,
    txs: Vec<SignedTx>,
}

impl Block {
    pub fn new(
        parent: H256,
        number: u64,
        nonce: u64,
        time: u64,
        miner: &str,
        txs: Vec<SignedTx>,
    ) -> Self {
        Self {
            header: BlockHeader {
                parent: parent,
                number: number,
                nonce: nonce,
                time: time,
                miner: miner.to_owned(),
            },
            txs: txs,
        }
    }

    pub fn update_nonce(mut self, nonce: u64) -> Self {
        self.header.nonce = nonce;
        self
    }

    pub fn hash(&self) -> H256 {
        let encoded = serde_json::to_string(self).unwrap();
        hash_message(encoded)
    }

    pub fn gas_reward(&self) -> u64 {
        self.txs.iter().map(|tx| tx.gas_cost()).sum()
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct BlockHeader {
    parent: H256,
    number: u64,
    nonce: u64,
    time: u64,
    miner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockKV {
    key: H256,
    value: Block,
}

impl BlockKV {
    pub fn take_hash(&mut self) -> H256 {
        mem::take(&mut self.key)
    }

    pub fn take_block(&mut self) -> Block {
        mem::take(&mut self.value)
    }
}
