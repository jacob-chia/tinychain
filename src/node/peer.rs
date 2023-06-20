use crate::error::Error;

use super::{Block, SignedTx};

/// Peer is a trait that defines the interface for a peer.
pub trait Peer {
    /// Return the peers (base58 encoded peer ids) that this node knows about.
    fn known_peers(&self) -> Vec<String>;

    /// Get the block height from a peer.
    fn get_block_height(&self, peer_id: &str) -> Result<u64, Error>;

    /// Get blocks from a peer, starting from the `from_number`.
    fn get_blocks(&self, peer_id: &str, from_number: u64) -> Result<Vec<Block>, Error>;

    /// Broadcast a transaction to the network.
    fn broadcast_tx(&self, tx: SignedTx);

    /// Broadcast a block to the network.
    fn broadcast_block(&self, block: Block);
}
