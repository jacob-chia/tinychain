use std::collections::HashMap;

use crate::{biz::State, error::Error, schema::Block, types::Hash};

#[derive(Debug, Clone)]
pub struct MemoryState;

impl State for MemoryState {
    fn block_height(&self) -> u64 {
        todo!()
    }

    fn next_account_nonce(&self, _account: &str) -> u64 {
        todo!()
    }

    fn last_block(&self) -> Option<Block> {
        todo!()
    }

    fn add_block(&self, _block: Block) -> Result<Hash, Error> {
        todo!()
    }

    fn get_blocks(&self, _from_number: u64) -> Vec<Block> {
        todo!()
    }

    fn get_block(&self, _number: u64) -> Option<Block> {
        todo!()
    }

    fn get_balance(&self, _account: &str) -> u64 {
        todo!()
    }

    fn get_balances(&self) -> HashMap<String, u64> {
        todo!()
    }
}
