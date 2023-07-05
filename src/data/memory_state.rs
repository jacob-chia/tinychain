use std::{
    collections::{BTreeMap, HashMap},
    sync::{Arc, RwLock},
};

use crate::{biz::State, error::Error, schema::Block, types::Hash};

#[derive(Debug, Clone)]
pub struct MemoryState {
    inner: Arc<RwLock<InnerState>>,
}

#[derive(Debug, Clone)]
struct InnerState {
    blocks: BTreeMap<u64, Block>,
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
}

impl MemoryState {
    pub fn new(balances: HashMap<String, u64>) -> Self {
        let inner = InnerState {
            blocks: BTreeMap::new(),
            balances,
            account2nonce: HashMap::new(),
        };

        Self {
            inner: Arc::new(RwLock::new(inner)),
        }
    }
}

impl State for MemoryState {
    fn block_height(&self) -> u64 {
        self.inner.read().unwrap().blocks.len() as u64
    }

    fn next_account_nonce(&self, account: &str) -> u64 {
        self.inner
            .read()
            .unwrap()
            .account2nonce
            .get(account)
            .cloned()
            .unwrap_or(0)
    }

    fn last_block_hash(&self) -> Option<Hash> {
        self.inner
            .read()
            .unwrap()
            .blocks
            .values()
            .last()
            .map(|b| b.hash())
    }

    fn add_block(&self, block: Block) -> Result<(), Error> {
        self.inner
            .write()
            .unwrap()
            .blocks
            .insert(block.number(), block);

        Ok(())
    }

    fn get_blocks(&self, from_number: u64) -> Vec<Block> {
        self.inner
            .read()
            .unwrap()
            .blocks
            .range(from_number..)
            .map(|(_, block)| block.clone())
            .collect()
    }

    fn get_block(&self, number: u64) -> Option<Block> {
        self.inner.read().unwrap().blocks.get(&number).cloned()
    }

    fn get_balance(&self, account: &str) -> u64 {
        self.inner
            .read()
            .unwrap()
            .balances
            .get(account)
            .cloned()
            .unwrap_or(0)
    }

    fn get_balances(&self) -> HashMap<String, u64> {
        self.inner.read().unwrap().balances.clone()
    }

    fn get_account2nonce(&self) -> HashMap<String, u64> {
        self.inner.read().unwrap().account2nonce.clone()
    }
}
