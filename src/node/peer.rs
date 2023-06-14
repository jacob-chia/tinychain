use crate::error::Error;

use super::{Block, SignedTx};

/// Peer is a trait that defines the interface for a peer.
pub trait Peer {
    /// Return the peers (base58 encoded peer ids) that this node knows about.
    fn known_peers(&self) -> Vec<String>;

    /// Get the best block number from a peer.
    /// Ok(None) indicates that there is no block yet in the peer.
    fn get_best_number(&self, peer_id: &str) -> Result<Option<u64>, Error>;

    /// Get the blocks from a peer.
    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error>;

    /// Broadcast a transaction to the network.
    fn broadcast_tx(&self, tx: SignedTx);

    /// Broadcast a block to the network.
    fn broadcast_block(&self, block: Block);
}
