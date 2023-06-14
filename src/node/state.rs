use std::collections::HashMap;

use super::Block;
use crate::{error::Error, types::Hash};

/// State is a trait that defines the interface for a state machine.
pub trait State {
    /// Get all the balances of the accounts.
    fn get_balances(&self) -> HashMap<String, u64>;

    /// Next block number to be mined.
    fn next_block_number(&self) -> u64;

    /// Next account nonce to be used.
    fn next_account_nonce(&self, account: &str) -> u64;

    /// Get the latest block.
    fn latest_block(&self) -> Option<Block>;

    /// Get the latest block hash.
    fn latest_block_hash(&self) -> Option<Hash>;

    /// Get the latest block number.
    fn latest_block_number(&self) -> Option<u64>;

    /// Add a block to the state.
    fn add_block(&mut self, block: Block) -> Result<Hash, Error>;

    /// Get the blocks from a number.
    fn get_blocks(&self, from_number: u64) -> Result<Vec<Block>, Error>;

    /// Get a block by its number.
    fn get_block(&self, number: u64) -> Result<Block, Error>;

    /// Get the mining difficulty.
    fn get_mining_difficulty(&self) -> usize;
}
