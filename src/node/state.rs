use std::collections::HashMap;

use super::Block;
use crate::{error::ChainError, types::Hash};

pub trait State {
    fn get_balances(&self) -> HashMap<String, u64>;
    fn next_block_number(&self) -> u64;
    fn next_account_nonce(&self, account: &str) -> u64;
    fn latest_block(&self) -> Block;
    fn latest_block_hash(&self) -> Hash;
    fn latest_block_number(&self) -> u64;
    fn add_block(&mut self, block: Block) -> Result<Hash, ChainError>;
    fn get_blocks(&self, offset: usize) -> Result<Vec<Block>, ChainError>;
    fn get_block(&self, number: u64) -> Result<Block, ChainError>;
    fn get_mining_difficulty(&self) -> usize;
}
