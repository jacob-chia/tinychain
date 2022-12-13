use super::SignedTx;
use crate::{types::Hash, utils};

struct PendingBlock {
    parent: Hash,
    number: u64,
    time: u64,
    miner: String,
    txs: Vec<SignedTx>,
}

impl PendingBlock {
    pub fn new(parent: Hash, number: u64, miner: String, txs: Vec<SignedTx>) -> Self {
        Self {
            parent: parent,
            number: number,
            time: utils::unix_timestamp(),
            miner: miner,
            txs: txs,
        }
    }
}
