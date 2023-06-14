use std::mem;

use serde::{Deserialize, Serialize};

use super::SignedTx;
use crate::{types::Hash, utils};

const BLOCK_REWORD: u64 = 100;

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct BlockHeader {
    pub parent_hash: Hash,
    pub number: u64,
    pub nonce: u64,
    pub timestamp: u64,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct Block {
    pub header: BlockHeader,
    pub txs: Vec<SignedTx>,
}

impl<'a> Block {
    pub fn builder() -> BlockBuilder<'a> {
        BlockBuilder::default()
    }

    pub fn update_nonce_and_time(&mut self) {
        self.header.nonce = utils::gen_random_number();
        self.header.timestamp = utils::unix_timestamp();
    }

    pub fn hash(&self) -> Hash {
        let encoded = serde_json::to_string(self).unwrap();
        utils::hash_message(&encoded)
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
                parent_hash: self.parent,
                nonce: self.nonce,
                timestamp: self.time,
                author: self.miner.to_owned(),
            },
            txs: self.txs,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockKV {
    pub key: Hash,
    pub value: Block,
}

impl BlockKV {
    pub fn take_block(&mut self) -> Block {
        mem::take(&mut self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_builder_works() {
        let mut block = Block::builder().number(1).nonce(1).time(1).build();

        assert_eq!(block.header.number, 1);
        assert_eq!(block.header.nonce, 1);
        assert_eq!(block.header.timestamp, 1);
        assert_eq!(block.txs.len(), 0);
        assert_eq!(block.block_reward(), 100);
        assert_eq!(block.hash().len(), 32);

        block.update_nonce_and_time();
        assert_ne!(block.header.nonce, 1);
        assert_ne!(block.header.timestamp, 1);
    }
}
