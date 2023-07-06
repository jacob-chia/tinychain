use std::{collections::HashMap, fmt::Debug};

use crate::{error::Error, schema::Block, types::Hash};

pub trait State: Debug + Clone + Send + Sync + 'static {
    /// Current block height.
    fn block_height(&self) -> u64;

    /// Next account nonce to be used.
    fn next_account_nonce(&self, account: &str) -> u64;

    /// Get the last block hash.
    fn last_block_hash(&self) -> Option<Hash>;

    /// Add a block to the state.
    fn add_block(&self, block: Block) -> Result<(), Error>;

    /// Get blocks, starting from the `from_number`.
    fn get_blocks(&self, from_number: u64) -> Vec<Block>;

    /// Get a block by its number.
    fn get_block(&self, number: u64) -> Option<Block>;

    /// Get the balance of the account.
    fn get_balance(&self, account: &str) -> u64;

    /// Get all the balances.
    fn get_balances(&self) -> HashMap<String, u64>;

    /// Get all the nonces of the accounts.
    fn get_account2nonce(&self) -> HashMap<String, u64>;
}
