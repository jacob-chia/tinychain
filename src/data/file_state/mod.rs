use log::info;
use once_cell::sync::OnceCell;
use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
};

use crate::{
    error::ChainError,
    node::{Block, BlockKV, SignedTx, State},
    types::Hash,
    utils,
};

mod genesis;

pub use genesis::*;

static DATABASE_DIR: OnceCell<String> = OnceCell::new();
static GENESIS_PATH: OnceCell<String> = OnceCell::new();
static BLOCKDB_PATH: OnceCell<String> = OnceCell::new();

pub fn init_database_dir(datadir: &str) {
    let mut dir = datadir.to_owned();
    dir.push_str("database/");

    let mut genesis_path = dir.clone();
    let mut blockdb_path = genesis_path.clone();
    genesis_path.push_str("genesis.json");
    blockdb_path.push_str("block.db");

    DATABASE_DIR.get_or_init(|| dir);
    GENESIS_PATH.get_or_init(|| genesis_path);
    BLOCKDB_PATH.get_or_init(|| blockdb_path);
}

#[derive(Debug, Clone, Default)]
pub struct FileState {
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
    latest_block: Block,
    latest_block_hash: Hash,
    mining_difficulty: usize,
    has_blocks: bool,
}

impl FileState {
    pub fn new(mining_difficulty: usize) -> Result<Self, ChainError> {
        let genesis = Genesis::load()?;
        info!("Genesis loaded, token symbol: {}", genesis.symbol);

        let mut state = Self {
            balances: genesis.clone_balances(),
            mining_difficulty: mining_difficulty,
            ..Default::default()
        };

        state.load_db()?;
        Ok(state)
    }

    fn load_db(&mut self) -> Result<(), ChainError> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = OpenOptions::new().read(true).open(db_path)?;
        let lines = BufReader::new(db).lines();

        for line in lines {
            if let Ok(ref block_str) = line {
                let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
                let block = block_kv.take_block();
                self.apply_block(block)?;
            } else {
                info!("load_db error: {:?}", line);
            }
        }

        Ok(())
    }

    fn persist(&mut self) -> Result<(), ChainError> {
        let mut block_json = serde_json::to_string(&BlockKV {
            key: self.latest_block_hash,
            value: self.latest_block.clone(),
        })?;
        block_json.push('\n');

        let db_path = BLOCKDB_PATH.get().unwrap();
        OpenOptions::new()
            .append(true)
            .open(db_path)?
            .write_all(block_json.as_bytes())?;

        Ok(())
    }

    fn apply_block(&mut self, block: Block) -> Result<(), ChainError> {
        self.check_block(&block)?;
        self.apply_txs(&block.txs)?;

        *self
            .balances
            .entry(block.header.miner.to_owned())
            .or_default() += block.block_reward();

        self.latest_block_hash = block.hash();
        self.latest_block = block;
        self.has_blocks = true;

        Ok(())
    }

    fn check_block(&self, block: &Block) -> Result<(), ChainError> {
        // check number
        let expected_number = self.latest_block.header.number + 1;
        if self.has_blocks && expected_number != block.header.number {
            return Err(ChainError::InvalidBlockNumber(
                expected_number,
                block.header.number,
            ));
        }

        // check header
        if self.has_blocks && self.latest_block_hash != block.header.parent {
            return Err(ChainError::InvalidBlockParent(
                self.latest_block_hash,
                block.header.parent,
            ));
        }

        // check hash
        if !self.is_valid_hash(&block.hash()) {
            return Err(ChainError::InvalidBlockHash(
                block.hash(),
                self.mining_difficulty,
            ));
        }

        Ok(())
    }

    fn is_valid_hash(&self, hash: &Hash) -> bool {
        utils::is_valid_hash(hash, self.mining_difficulty)
    }

    fn apply_txs(&mut self, signed_txs: &[SignedTx]) -> Result<(), ChainError> {
        for signed_tx in signed_txs {
            self.check_tx(signed_tx)?;

            let tx = &signed_tx.tx;
            *self.balances.get_mut(&tx.from).unwrap() -= tx.cost();
            *self.balances.entry(tx.to.to_owned()).or_default() += tx.value;
            *self.account2nonce.entry(tx.from.to_owned()).or_default() = tx.nonce;
        }

        Ok(())
    }

    fn check_tx(&self, signed_tx: &SignedTx) -> Result<(), ChainError> {
        signed_tx.check_signature()?;

        let tx = &signed_tx.tx;

        // check account nonce
        let expected_nonce = self.next_account_nonce(&tx.from);
        if expected_nonce != tx.nonce {
            return Err(ChainError::InvalidTxNonce(expected_nonce, tx.nonce));
        }

        // check balance
        let balance = *self.balances.get(&tx.from).unwrap_or(&0);
        if balance < tx.cost() {
            return Err(ChainError::InsufficientBalance(tx.cost(), balance));
        }

        Ok(())
    }
}

impl State for FileState {
    fn get_balances(&self) -> HashMap<String, u64> {
        self.balances.clone()
    }

    fn next_block_number(&self) -> u64 {
        if self.has_blocks {
            return self.latest_block.header.number + 1;
        }

        0
    }

    fn next_account_nonce(&self, account: &str) -> u64 {
        *self.account2nonce.get(account).unwrap_or(&0) + 1
    }

    fn latest_block(&self) -> Block {
        self.latest_block.clone()
    }

    fn latest_block_hash(&self) -> Hash {
        self.latest_block_hash
    }

    fn latest_block_number(&self) -> u64 {
        if self.has_blocks {
            return self.latest_block.header.number;
        }

        0
    }

    fn add_block(&mut self, block: Block) -> Result<Hash, ChainError> {
        // Why clone?
        // To prevent the state from being corrupted by invalid blocks.
        let mut state = self.clone();
        state.apply_block(block)?;
        state.persist()?;
        *self = state;

        Ok(self.latest_block_hash)
    }

    fn get_blocks(&self, offset: usize) -> Result<Vec<Block>, ChainError> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = OpenOptions::new().read(true).open(db_path)?;

        Ok(BufReader::new(db)
            .lines()
            .skip(offset)
            .map(|line| {
                serde_json::from_str::<BlockKV>(&line.unwrap())
                    .unwrap()
                    .take_block()
            })
            .collect::<Vec<_>>())
    }

    fn get_block(&self, number: u64) -> Result<Block, ChainError> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = OpenOptions::new().read(true).open(db_path)?;

        BufReader::new(db)
            .lines()
            .nth(number as usize)
            .map(|line| {
                serde_json::from_str::<BlockKV>(&line.unwrap())
                    .unwrap()
                    .take_block()
            })
            .ok_or(ChainError::BlockNotFound(number))
    }

    fn get_mining_difficulty(&self) -> usize {
        self.mining_difficulty
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_dir_can_only_be_initialized_once() {
        init_database_dir("/tmp/");
        assert_eq!("/tmp/database/", DATABASE_DIR.get().unwrap());
        assert_eq!("/tmp/database/genesis.json", GENESIS_PATH.get().unwrap());
        assert_eq!("/tmp/database/block.db", BLOCKDB_PATH.get().unwrap());

        init_database_dir("/another/dir/");
        assert_eq!("/tmp/database/", DATABASE_DIR.get().unwrap());
        assert_eq!("/tmp/database/genesis.json", GENESIS_PATH.get().unwrap());
        assert_eq!("/tmp/database/block.db", BLOCKDB_PATH.get().unwrap());
    }
}
