use anyhow::Result;
use ethers_core::types::H256;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use super::*;

#[derive(Debug, Default)]
pub struct State {
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
    latest_block: Block,
    latest_block_hash: H256,
    mining_difficulty: i32,
    has_blocks: bool,
}

impl State {
    pub fn new(mining_difficulty: i32) -> Result<Self> {
        let genesis = Genesis::load()?;

        let mut state = Self {
            balances: genesis.clone_balances(),
            mining_difficulty: mining_difficulty,
            ..Default::default()
        };

        state.load_db()?;
        Ok(state)
    }

    fn load_db(&mut self) -> Result<()> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = File::open(db_path)?;
        let lines = BufReader::new(db).lines();

        for line in lines {
            if let Ok(ref block_str) = line {
                self.apply_block(block_str)?;
            }
        }

        Ok(())
    }

    fn apply_block(&mut self, block_str: &str) -> Result<()> {
        let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
        let hash = block_kv.take_hash();
        let block = block_kv.take_block();

        Ok(())
    }
}
