use std::time::Duration;

use crossbeam_channel::{select, tick};

use super::*;
use crate::{types::Hash, utils};

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn mine(&self) {
        let ticker = tick(Duration::from_secs(5));

        loop {
            select! {
                recv(ticker) -> _ => {
                }
            }
        }
    }
}

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
