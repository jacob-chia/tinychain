use serde::Deserialize;

use crate::{error::ChainError, types::Hash};

use super::{Block, SignedTx};

#[derive(Debug, Deserialize)]
pub struct PeerStatus {
    pub hash: Hash,
    pub number: u64,
    pub peers: Vec<String>,
    pub pending_txs: Vec<SignedTx>,
}

pub trait Peer {
    fn ping(&self, my_addr: &str, peer_addr: &str) -> Result<(), ChainError>;
    fn get_status(&self, peer_addr: &str) -> Result<PeerStatus, ChainError>;
    fn get_blocks(&self, peer_addr: &str, offset: u64) -> Result<Vec<Block>, ChainError>;
}
