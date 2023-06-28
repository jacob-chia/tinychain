use std::fmt::Debug;

use crate::error::Error;

use super::{Block, SignedTx};

/// PeerClient is a trait that defines the behaviour of a peer client.
pub trait PeerClient: Debug + Clone + Send + Sync + 'static {
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
