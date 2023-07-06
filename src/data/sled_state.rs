use std::collections::HashMap;

use sled::{
    self,
    transaction::{
        ConflictableTransactionResult, TransactionError, Transactional, TransactionalTree,
    },
};

use crate::{biz::State, error::Error, schema::Block, types::Hash};

#[derive(Debug, Clone)]
pub struct SledState {
    blocks: sled::Tree,
    balances: sled::Tree,
    account2nonce: sled::Tree,
}

impl SledState {
    /// Create a new `SledState` instance.
    pub fn new(data_dir: &str, balances: HashMap<String, u64>) -> Result<Self, Error> {
        let db = sled::open(data_dir)?;
        let state = Self {
            blocks: db.open_tree("blocks")?,
            balances: db.open_tree("balances")?,
            account2nonce: db.open_tree("account2nonce")?,
        };

        if state.balances.is_empty() {
            state.init_balances(balances);
        }

        Ok(state)
    }

    fn init_balances(&self, balances: HashMap<String, u64>) {
        for (account, balance) in balances {
            self.balances
                .insert(account.as_bytes(), u64_encode(balance))
                .unwrap();
        }
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
            .map_or(0, |bytes| u64_decode(&bytes))
    }

    fn last_block_hash(&self) -> Option<Hash> {
        self.blocks
            .last()
            .unwrap_or_default()
            .map(|(_, b)| Block::try_from(b.to_vec()).unwrap().hash())
    }

    fn add_block(&self, block: Block) -> Result<(), Error> {
        (&self.blocks, &self.balances, &self.account2nonce)
            .transaction(|(blocks, balances, account2nonce)| {
                // Apply txs
                for tx in &block.txs {
                    fetch_sub(balances, &tx.from, tx.cost())?;
                    fetch_add(balances, &tx.to, tx.value)?;
                    fetch_add(account2nonce, &tx.from, 1)?;
                }

                // Apply block
                fetch_add(balances, block.author(), block.block_reward())?;
                blocks.insert(u64_encode(block.number()), Vec::from(&block))?;

                Ok(())
            })
            .map_err(|e| match e {
                TransactionError::Abort(e) => e,
                _ => Error::AddBlockFailure,
            })?;

        Ok(())
    }

    fn get_blocks(&self, from_number: u64) -> Vec<Block> {
        let start = u64_encode(from_number);

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
            .get(u64_encode(number))
            .unwrap_or_default()
            .map(|block| Block::try_from(block.to_vec()).unwrap())
    }

    fn get_balance(&self, account: &str) -> u64 {
        self.balances
            .get(account)
            .unwrap_or_default()
            .map_or(0, |bytes| u64_decode(&bytes))
    }

    fn get_balances(&self) -> HashMap<String, u64> {
        self.balances
            .iter()
            .map(|result| {
                let (account, balance) = result.unwrap();
                (
                    String::from_utf8(account.to_vec()).unwrap(),
                    u64_decode(&balance),
                )
            })
            .collect()
    }

    fn get_account2nonce(&self) -> HashMap<String, u64> {
        self.account2nonce
            .iter()
            .map(|result| {
                let (account, nonce) = result.unwrap();
                (
                    String::from_utf8(account.to_vec()).unwrap(),
                    u64_decode(&nonce),
                )
            })
            .collect()
    }
}

fn fetch_add(
    tree: &TransactionalTree,
    key: &str,
    num: u64,
) -> ConflictableTransactionResult<(), Error> {
    let key = key.as_bytes();
    let old = get_u64(tree, key).unwrap_or_default();
    let new = u64_encode(old + num);
    tree.insert(key, new)?;

    Ok(())
}

fn fetch_sub(
    tree: &TransactionalTree,
    key: &str,
    num: u64,
) -> ConflictableTransactionResult<(), Error> {
    let key = key.as_bytes();
    let old = get_u64(tree, key).unwrap_or_default();
    match old - num {
        0 => tree.remove(key)?,
        n => tree.insert(key, u64_encode(n))?,
    };

    Ok(())
}

fn get_u64(tree: &TransactionalTree, key: &[u8]) -> Option<u64> {
    tree.get(key)
        .unwrap_or_default()
        .map(|v| u64_decode(v.as_ref()))
}

fn u64_decode(bytes: &[u8]) -> u64 {
    let mut buf = [0u8; 8];
    buf.copy_from_slice(bytes);
    u64::from_be_bytes(buf)
}

fn u64_encode(n: u64) -> Vec<u8> {
    n.to_be_bytes().to_vec()
}
