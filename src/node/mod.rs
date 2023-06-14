//! The core logic of the blockchain node.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crossbeam_channel::Sender;
use dashmap::DashMap;
use log::error;

use crate::{error::Error, types::Hash};

mod block;
mod miner;
mod peer;
mod state;
mod syncer;
mod tx;

pub use block::*;
pub use peer::*;
pub use state::*;
pub use tx::*;

#[derive(Debug)]
pub struct Node<S, P> {
    /// The miner address of the node.
    pub miner: String,
    /// The pending transactions that are not yet included in a block.
    pub pending_txs: DashMap<Hash, SignedTx>,
    /// The mining difficulty of the node.
    pub mining_difficulty: usize,

    // A state machine that holds the state of the blockchain.
    pub state: Arc<RwLock<S>>,
    // A proxy to interact with peers, which is initialized after the node is created.
    pub peer_proxy: P,

    // A channel to send a signal to the miner to stop mining.
    pub cancel_signal_s: Sender<()>,
}

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    /// Create a new node with the given miner address and state.
    pub fn new(
        miner: String,
        state: S,
        peer: P,
        cancel_signal_s: Sender<()>,
    ) -> Result<Self, Error> {
        let node = Self {
            miner,
            pending_txs: DashMap::new(),
            mining_difficulty: state.get_mining_difficulty(),
            state: Arc::new(RwLock::new(state)),
            peer_proxy: peer,
            cancel_signal_s,
        };

        Ok(node)
    }

    /// Get the next nouce of the given account.
    /// The nounce is a monotonically increasing number that is used to prevent replay attacks.
    pub fn next_account_nonce(&self, account: &str) -> u64 {
        self.state.read().unwrap().next_account_nonce(account)
    }

    /// Transfer the given value from one account to another.
    pub fn transfer(&self, from: &str, to: &str, value: u64, nonce: u64) -> Result<(), Error> {
        let tx = Tx::builder()
            .from(from)
            .to(to)
            .value(value)
            .nonce(nonce)
            .build()
            .sign()?;

        self.add_pending_tx(tx.clone())?;
        self.peer_proxy.broadcast_tx(tx);

        Ok(())
    }

    /// Get blocks from the given number.
    pub fn get_blocks(&self, from_number: u64) -> Result<Vec<Block>, Error> {
        self.state.read().unwrap().get_blocks(from_number)
    }

    /// Get the block with the given number.
    pub fn get_block(&self, number: u64) -> Result<Block, Error> {
        self.state.read().unwrap().get_block(number)
    }

    /// Get all the balances of the accounts.
    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.read().unwrap().get_balances()
    }

    /// Get the latest block hash.
    pub fn latest_block_hash(&self) -> Option<Hash> {
        self.state.read().unwrap().latest_block_hash()
    }

    /// Get the latest block number.
    pub fn latest_block_number(&self) -> Option<u64> {
        self.state.read().unwrap().latest_block_number()
    }

    /// Get the next block number.
    pub fn next_block_number(&self) -> u64 {
        self.state.read().unwrap().next_block_number()
    }

    /// Add a block to the blockchain.
    pub fn add_block(&self, block: Block) -> bool {
        if let Err(err) = self.state.write().unwrap().add_block(block.clone()) {
            error!("❌ Failed to add block: {:?}", err);
            self.remove_invalid_txs(err);
            return false;
        }

        self.remove_mined_txs(&block);
        true
    }

    /// Add a pending transaction to the transaction pool.
    pub fn add_pending_tx(&self, tx: SignedTx) -> Result<(), Error> {
        tx.check_signature()?;
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }

    fn remove_mined_txs(&self, block: &Block) {
        for tx in &block.txs {
            self.pending_txs.remove(&tx.hash());
        }
    }

    fn remove_invalid_txs(&self, err: Error) {
        let account = match err {
            Error::InvalidTxSignature(acc) => Some(acc),
            Error::InvalidTxNonce(acc, ..) => Some(acc),
            Error::InsufficientBalance(acc, ..) => Some(acc),
            _ => None,
        };

        // Remove all the transactions from the account that caused the error.
        if let Some(acc) = account {
            self.pending_txs.retain(|_, tx| tx.from != acc);
        }
    }
}
