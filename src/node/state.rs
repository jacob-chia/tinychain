use std::collections::HashMap;

use super::Block;
use crate::{error::Error, types::Hash};

/// State is a trait that defines the interface for a state machine.
pub trait State {
    /// Current block height.
    fn block_height(&self) -> u64;

    /// Next account nonce to be used.
    fn next_account_nonce(&self, account: &str) -> u64;

    /// Get the last block.
    fn last_block(&self) -> Option<Block>;

    /// Add a block to the state.
    fn add_block(&self, block: Block) -> Result<Hash, Error>;

    /// Get blocks, starting from the `from_number`.
    fn get_blocks(&self, from_number: u64) -> Vec<Block>;

    /// Get a block by its number.
    fn get_block(&self, number: u64) -> Option<Block>;

    /// Get the balance of the account.
    fn get_balance(&self, account: &str) -> u64;

    /// Get all the balances for debugging.
    fn get_balances(&self) -> HashMap<String, u64>;
}
