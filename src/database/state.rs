use anyhow::{anyhow, Result};
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
    mining_difficulty: usize,
    has_blocks: bool,
}

impl State {
    pub fn new(mining_difficulty: usize) -> Result<Self> {
        let genesis = Genesis::load()?;

        let mut state = Self {
            balances: genesis.clone_balances(),
            mining_difficulty: mining_difficulty,
            ..Default::default()
        };

        state.load_db()?;
        Ok(state)
    }

    pub fn next_account_nonce(&self, account: &str) -> u64 {
        *self.account2nonce.get(account).unwrap_or(&0) + 1
    }

    fn load_db(&mut self) -> Result<()> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = File::open(db_path)?;
        let lines = BufReader::new(db).lines();

        for line in lines {
            if let Ok(ref block_str) = line {
                self.load_one_block(block_str)?;
            }
        }

        Ok(())
    }

    fn load_one_block(&mut self, block_str: &str) -> Result<()> {
        let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
        let hash = block_kv.take_hash();
        let block = block_kv.take_block();

        self.check_block(&block, &hash);
        // self.apply_txs(&block.txs)?;

        Ok(())
    }

    fn check_block(&self, block: &Block, hash: &H256) -> Result<()> {
        // check number
        let expected_number = self.latest_block.header.number + 1;
        if self.has_blocks && expected_number != block.header.number {
            return Err(anyhow!(
                "block number error: expected '{}', not '{}'",
                expected_number,
                block.header.number
            ));
        }

        // check header
        if self.has_blocks && self.latest_block_hash != block.header.parent {
            return Err(anyhow!(
                "block parent error: expected '{}', not '{}'",
                self.latest_block_hash,
                block.header.parent
            ));
        }

        // check hash
        if hash[..] != block.hash()[..] || !self.is_valid_hash(hash) {
            return Err(anyhow!("invalid block hash: '{hash}'"));
        }

        Ok(())
    }

    fn is_valid_hash(&self, hash: &H256) -> bool {
        let hash_prefix = vec![0u8; self.mining_difficulty];
        hash_prefix[..] == hash[..self.mining_difficulty]
    }

    fn apply_txs(&mut self, signed_txs: &[SignedTx]) -> Result<()> {
        for signed_tx in signed_txs {
            self.check_tx(signed_tx)?;
        }

        Ok(())
    }

    fn check_tx(&self, signed_tx: &SignedTx) -> Result<()> {
        // check signature
        if !signed_tx.is_valid_signature() {
            return Err(anyhow!("invalid tx signature"));
        }
        // check account nonce

        Ok(())
    }
}
