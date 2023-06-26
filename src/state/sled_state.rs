use std::collections::HashMap;

use log::error;
use sled::{
    self,
    transaction::{
        ConflictableTransactionError, ConflictableTransactionResult, TransactionError,
        Transactional, TransactionalTree,
    },
};

use crate::{
    error::Error,
    node::State,
    schema::{Block, SignedTx},
    types::Hash,
    utils,
};

#[derive(Debug, Clone)]
pub struct SledState {
    blocks: sled::Tree,
    balances: sled::Tree,
    account2nonce: sled::Tree,

    // mining_difficulty is used to check the validity of a block.
    mining_difficulty: usize,
}

impl SledState {
    /// Create a new `SledState` instance.
    pub fn new(
        data_dir: &str,
        balances: HashMap<String, u64>,
        mining_difficulty: usize,
    ) -> Result<Self, Error> {
        let db = sled::open(data_dir)?;
        let state = Self {
            blocks: db.open_tree("blocks")?,
            balances: db.open_tree("balances")?,
            account2nonce: db.open_tree("account2nonce")?,
            mining_difficulty,
        };

        if state.balances.is_empty() {
            state.init_balances(balances);
        }

        Ok(state)
    }

    fn init_balances(&self, balances: HashMap<String, u64>) {
        for (account, balance) in balances {
            self.balances
                .insert(account.as_bytes(), Self::u64_encode(balance))
                .unwrap();
        }
    }

    fn fetch_add(
        tree: &TransactionalTree,
        key: &[u8],
        num: u64,
    ) -> ConflictableTransactionResult<(), Error> {
        let old = Self::get_u64(tree, key).unwrap_or_default();
        let new = Self::u64_encode(old + num);
        tree.insert(key, new)?;

        Ok(())
    }

    fn fetch_sub(
        tree: &TransactionalTree,
        key: &[u8],
        num: u64,
    ) -> ConflictableTransactionResult<(), Error> {
        let old = Self::get_u64(tree, key).unwrap_or_default();
        match old - num {
            0 => tree.remove(key)?,
            n => tree.insert(key, Self::u64_encode(n))?,
        };

        Ok(())
    }

    fn get_u64(tree: &TransactionalTree, key: &[u8]) -> Option<u64> {
        tree.get(key)
            .unwrap_or_default()
            .map(|v| Self::u64_decode(v.as_ref()))
    }

    fn u64_decode(bytes: &[u8]) -> u64 {
        let mut buf = [0u8; 8];
        buf.copy_from_slice(bytes);
        u64::from_be_bytes(buf)
    }

    fn u64_encode(n: u64) -> Vec<u8> {
        n.to_be_bytes().to_vec()
    }

    fn check_block(
        &self,
        last_block: &Option<Block>,
        block: &Block,
    ) -> ConflictableTransactionResult<(), Error> {
        let expected_number = last_block
            .as_ref()
            .map(|b| b.number() + 1)
            .unwrap_or_default();
        if block.number() != expected_number {
            let err = Error::InvalidBlockNumber(expected_number, block.number());
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        let expected_hash = last_block.as_ref().map(|b| b.hash()).unwrap_or_default();
        if block.parent_hash() != expected_hash {
            let err = Error::InvalidBlockParent(expected_hash, block.parent_hash());
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        if !utils::is_valid_hash(&block.hash(), self.mining_difficulty) {
            let err = Error::InvalidBlockHash(block.hash(), self.mining_difficulty);
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        Ok(())
    }

    fn check_tx(
        balances: &TransactionalTree,
        account2nonce: &TransactionalTree,
        tx: &SignedTx,
    ) -> ConflictableTransactionResult<(), Error> {
        if let Err(err) = utils::verify_tx(tx) {
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        let expected_nonce = Self::get_u64(account2nonce, tx.from.as_bytes()).unwrap_or_default();
        if tx.nonce != expected_nonce {
            let err = Error::InvalidTxNonce(tx.from.clone(), expected_nonce, tx.nonce);
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        let from_balance = Self::get_u64(balances, tx.from.as_bytes()).unwrap_or_default();
        if from_balance < tx.cost() {
            let err = Error::BalanceInsufficient(tx.from.clone(), from_balance, tx.cost());
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        let to_balance = Self::get_u64(balances, tx.to.as_bytes()).unwrap_or_default();
        if to_balance.checked_add(tx.value).is_none() {
            let err = Error::BalanceOverflow(tx.to.clone(), to_balance, tx.value);
            error!("❌ {}", err.to_string());
            return Err(ConflictableTransactionError::Abort(err));
        }

        Ok(())
    }
}

impl State for SledState {
    fn block_height(&self) -> u64 {
        self.blocks.len() as u64
    }

    fn next_account_nonce(&self, account: &str) -> u64 {
        self.account2nonce
            .get(account)
            .unwrap_or_default()
            .map_or(0, |bytes| Self::u64_decode(&bytes))
    }

    fn last_block(&self) -> Option<Block> {
        self.blocks
            .last()
            .unwrap_or_default()
            .map(|(_, block)| Block::try_from(block.to_vec()).unwrap())
    }

    fn add_block(&self, block: Block) -> Result<Hash, Error> {
        let last_block = self.last_block();

        (&self.blocks, &self.balances, &self.account2nonce)
            .transaction(|(blocks, balances, account2nonce)| {
                // Check the block before applying
                self.check_block(&last_block, &block)?;

                // Apply txs
                for tx in &block.txs {
                    // Check the tx before applying
                    Self::check_tx(balances, account2nonce, tx)?;

                    Self::fetch_sub(balances, tx.from.as_bytes(), tx.cost())?;
                    Self::fetch_add(balances, tx.to.as_bytes(), tx.value)?;
                    Self::fetch_add(account2nonce, tx.from.as_bytes(), 1)?;
                }

                // Apply block
                let author = &block.header.as_ref().unwrap().author;
                Self::fetch_add(balances, author.as_bytes(), block.block_reward())?;
                blocks.insert(Self::u64_encode(block.number()), Vec::from(&block))?;

                Ok(())
            })
            .map_err(|e| match e {
                TransactionError::Abort(e) => e,
                _ => Error::AddBlockFailure,
            })?;

        Ok(block.hash())
    }

    fn get_blocks(&self, from_number: u64) -> Vec<Block> {
        let start = Self::u64_encode(from_number);

        self.blocks
            .range(start..)
            .map(|result| {
                let (_, block) = result.unwrap();
                Block::try_from(block.to_vec()).unwrap()
            })
            .collect()
    }

    fn get_block(&self, number: u64) -> Option<Block> {
        self.blocks
            .get(Self::u64_encode(number))
            .unwrap_or_default()
            .map(|block| Block::try_from(block.to_vec()).unwrap())
    }

    fn get_balance(&self, account: &str) -> u64 {
        self.balances
            .get(account)
            .unwrap_or_default()
            .map_or(0, |bytes| Self::u64_decode(&bytes))
    }

    fn get_balances(&self) -> HashMap<String, u64> {
        self.balances
            .iter()
            .map(|result| {
                let (account, balance) = result.unwrap();
                (
                    String::from_utf8(account.to_vec()).unwrap(),
                    Self::u64_decode(&balance),
                )
            })
            .collect()
    }
}
