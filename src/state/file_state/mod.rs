use std::{
    collections::HashMap,
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
};

use log::{info, warn};
use once_cell::sync::OnceCell;

use crate::{
    error::Error,
    node::{Block, BlockKV, SignedTx, State},
    types::Hash,
    utils,
};

mod genesis;

pub use genesis::*;

static DATABASE_DIR: OnceCell<PathBuf> = OnceCell::new();
static GENESIS_PATH: OnceCell<PathBuf> = OnceCell::new();
static BLOCKDB_PATH: OnceCell<PathBuf> = OnceCell::new();

pub fn init_database_dir(datadir: impl AsRef<Path>) {
    let mut dir = datadir.as_ref().to_path_buf();
    dir.push("database");

    let genesis_path = dir.join("genesis.json");
    let blockdb_path = dir.join("block.db");

    DATABASE_DIR.get_or_init(|| dir);
    GENESIS_PATH.get_or_init(|| genesis_path);
    BLOCKDB_PATH.get_or_init(|| blockdb_path);
}

#[derive(Debug, Clone, Default)]
pub struct FileState {
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
    latest_block: Option<Block>,
    latest_block_hash: Option<Hash>,
    mining_difficulty: usize,
}

impl FileState {
    pub fn new(mining_difficulty: usize) -> Result<Self, Error> {
        let genesis = Genesis::load()?;
        info!("📣 Genesis loaded, token symbol: {}", genesis.symbol);

        let mut state = Self {
            balances: genesis.clone_balances(),
            mining_difficulty,
            ..Default::default()
        };

        state.load_db()?;
        Ok(state)
    }

    fn load_db(&mut self) -> Result<(), Error> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = OpenOptions::new().read(true).open(db_path)?;
        let lines = BufReader::new(db).lines();

        for line in lines {
            if let Ok(ref block_str) = line {
                let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
                let block = block_kv.take_block();
                self.apply_block(block)?;
            } else {
                warn!("❗ load_db error: {:?}", line);
            }
        }

        Ok(())
    }

    fn persist(&mut self) -> Result<(), Error> {
        let mut block_json = serde_json::to_string(&BlockKV {
            key: self.latest_block_hash.unwrap_or_default(),
            value: self.latest_block.clone().unwrap_or_default(),
        })?;
        block_json.push('\n');

        let db_path = BLOCKDB_PATH.get().unwrap();
        OpenOptions::new()
            .append(true)
            .open(db_path)?
            .write_all(block_json.as_bytes())?;

        Ok(())
    }

    fn apply_block(&mut self, block: Block) -> Result<(), Error> {
        self.check_block(&block)?;
        self.apply_txs(&block.txs)?;

        *self
            .balances
            .entry(block.header.author.to_owned())
            .or_default() += block.block_reward();

        self.latest_block_hash = Some(block.hash());
        self.latest_block = Some(block);

        Ok(())
    }

    fn check_block(&self, block: &Block) -> Result<(), Error> {
        // check number
        let expected_number = self.next_block_number();
        if expected_number != block.header.number {
            return Err(Error::InvalidBlockNumber(
                expected_number,
                block.header.number,
            ));
        }

        // check header
        if let Some(latest_hash) = self.latest_block_hash {
            if block.header.parent_hash != latest_hash {
                return Err(Error::InvalidBlockParent(
                    latest_hash,
                    block.header.parent_hash,
                ));
            }
        }

        // check hash
        if !self.is_valid_hash(&block.hash()) {
            return Err(Error::InvalidBlockHash(
                block.hash(),
                self.mining_difficulty,
            ));
        }

        Ok(())
    }

    fn is_valid_hash(&self, hash: &Hash) -> bool {
        utils::is_valid_hash(hash, self.mining_difficulty)
    }

    fn apply_txs(&mut self, signed_txs: &[SignedTx]) -> Result<(), Error> {
        for tx in signed_txs {
            self.check_tx(tx)?;

            *self.balances.get_mut(&tx.from).unwrap() -= tx.cost();
            *self.balances.entry(tx.to.to_owned()).or_default() += tx.value;
            *self.account2nonce.entry(tx.from.to_owned()).or_default() = tx.nonce;
        }

        Ok(())
    }

    fn check_tx(&self, tx: &SignedTx) -> Result<(), Error> {
        tx.check_signature()?;

        // check account nonce
        let expected_nonce = self.next_account_nonce(&tx.from);
        if expected_nonce != tx.nonce {
            return Err(Error::InvalidTxNonce(
                tx.from.clone(),
                expected_nonce,
                tx.nonce,
            ));
        }

        // check balance
        let balance = *self.balances.get(&tx.from).unwrap_or(&0);
        if balance < tx.cost() {
            return Err(Error::InsufficientBalance(
                tx.from.clone(),
                tx.cost(),
                balance,
            ));
        }

        Ok(())
    }
}

impl State for FileState {
    fn get_balances(&self) -> HashMap<String, u64> {
        self.balances.clone()
    }

    fn next_block_number(&self) -> u64 {
        self.latest_block_number()
            .map(|number| number + 1)
            .unwrap_or_default()
    }

    fn next_account_nonce(&self, account: &str) -> u64 {
        *self.account2nonce.get(account).unwrap_or(&0) + 1
    }

    fn latest_block(&self) -> Option<Block> {
        self.latest_block.clone()
    }

    fn latest_block_hash(&self) -> Option<Hash> {
        self.latest_block_hash
    }

    fn latest_block_number(&self) -> Option<u64> {
        self.latest_block.as_ref().map(|block| block.header.number)
    }

    fn add_block(&mut self, block: Block) -> Result<Hash, Error> {
        // Apply the block to the cloned state to avoid mutating the original state if the block is invalid.
        let mut state = self.clone();
        state.apply_block(block)?;
        state.persist()?;
        *self = state;

        Ok(self.latest_block_hash.unwrap())
    }

    fn get_blocks(&self, from_number: u64) -> Result<Vec<Block>, Error> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = OpenOptions::new().read(true).open(db_path)?;

        Ok(BufReader::new(db)
            .lines()
            .skip(from_number as usize)
            .map(|line| {
                serde_json::from_str::<BlockKV>(&line.unwrap())
                    .unwrap()
                    .take_block()
            })
            .collect::<Vec<_>>())
    }

    fn get_block(&self, number: u64) -> Result<Block, Error> {
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
            .ok_or(Error::BlockNotFound(number))
    }

    fn get_mining_difficulty(&self) -> usize {
        self.mining_difficulty
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn test_database_dir_can_only_be_initialized_once() {
        let tmpdir = tempdir_with_prefix("tmp");
        let _ = fs::remove_dir_all(&tmpdir);

        init_database_dir(&tmpdir);
        let expected_db_dir = tmpdir.join("database");
        let expected_genesis = expected_db_dir.join("genesis.json");
        let expected_block_file = expected_db_dir.join("block.db");

        init_database_dir("/another/dir/");
        assert_eq!(&expected_db_dir, DATABASE_DIR.get().unwrap());
        assert_eq!(&expected_genesis, GENESIS_PATH.get().unwrap());
        assert_eq!(&expected_block_file, BLOCKDB_PATH.get().unwrap());
    }

    fn tempdir_with_prefix(prefix: &str) -> PathBuf {
        tempfile::Builder::new()
            .prefix(prefix)
            .tempdir()
            .unwrap()
            .into_path()
    }
}
