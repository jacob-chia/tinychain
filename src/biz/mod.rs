//! The entry point of the blockchain node.

use std::{collections::HashMap, ops::Deref, sync::Arc};

use crossbeam_channel::Sender;
use dashmap::DashMap;
use wallet::Wallet;

use crate::{error::Error, schema::*, types::Hash, utils};

mod genesis;
mod miner;
mod peer;
mod state;
mod syncer;

pub use genesis::*;
pub use peer::*;
pub use state::*;

#[derive(Debug, Clone)]
pub struct Node<S: State, P: Peer> {
    inner: Arc<NodeInner<S, P>>,
}

impl<S: State, P: Peer> Node<S, P> {
    /// Create a new node with the given miner address and state.
    pub fn new(
        author: String,
        state: S,
        peer: P,
        wallet: Wallet,
        cancel_signal_s: Sender<()>,
        mining_difficulty: usize,
    ) -> Result<Self, Error> {
        let inner = NodeInner {
            author,
            pending_txs: DashMap::new(),
            mining_difficulty,
            state,
            peer_proxy: peer,
            wallet,
            cancel_signal_s,
        };

        Ok(Self {
            inner: Arc::new(inner),
        })
    }
}

// Implement `Deref` so that `Node` can be treated as `NodeInner`.
impl<S: State, P: Peer> Deref for Node<S, P> {
    type Target = NodeInner<S, P>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[derive(Debug)]
pub struct NodeInner<S: State, P: Peer> {
    /// The author account of the node.
    author: String,
    /// The pending transactions that are not yet included in a block.
    pending_txs: DashMap<Hash, SignedTx>,
    /// The mining difficulty of the node.
    mining_difficulty: usize,

    // A state machine that holds the state of the blockchain.
    state: S,
    // A proxy to interact with peers, which is initialized after the node is created.
    peer_proxy: P,

    // For facilitating a smooth demonstration, the node holds a wallet that stores all
    // the keys of the users, so that it can sign transactions on behalf of the users.
    // In the real world, every user should have their own wallet.
    wallet: Wallet,

    // A channel to send a signal to the miner to stop mining.
    cancel_signal_s: Sender<()>,
}

impl<S: State, P: Peer> NodeInner<S, P> {
    /// Get the next nouce of the given account.
    /// The nounce is a monotonically increasing number that is used to prevent replay attacks.
    pub fn next_account_nonce(&self, account: &str) -> u64 {
        self.state.next_account_nonce(account)
    }

    /// Transfer the `value` from `from` to `to`. The `nonce` is used to prevent replay attacks.
    ///
    /// There may be multiple txs with the same `from`, and whether a tx is valid depends on
    /// the current state of the blockchain. It means that only if the previous tx has been
    /// applied to the state, can the next tx be checked, so we check the validity of the txs
    /// when we do `add_block`.
    pub fn transfer(&self, from: &str, to: &str, value: u64, nonce: u64) -> Result<(), Error> {
        let tx = Tx::new(from, to, value, nonce);
        let signed_tx = self.sign_tx(tx)?;

        self.add_pending_tx(signed_tx.clone())?;
        self.peer_proxy.broadcast_tx(signed_tx);
        Ok(())
    }

    /// Get blocks from the given number.
    pub fn get_blocks(&self, from_number: u64) -> Vec<Block> {
        self.state.get_blocks(from_number)
    }

    /// Get the block by the given number.
    pub fn get_block(&self, number: u64) -> Option<Block> {
        self.state.get_block(number)
    }

    /// Get all the balances of the accounts.
    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.get_balances()
    }

    /// Get the block height.
    pub fn block_height(&self) -> u64 {
        self.state.block_height()
    }

    /// Get the last block hash.
    pub fn last_block_hash(&self) -> Option<Hash> {
        self.state.last_block().map(|b| b.hash())
    }

    /// Add a block to the blockchain.
    pub fn add_block(&self, block: Block) -> Result<Hash, Error> {
        self.remove_mined_txs(&block);
        self.state.add_block(block)
    }

    /// Add a pending transaction to the transaction pool.
    pub fn add_pending_tx(&self, tx: SignedTx) -> Result<(), Error> {
        utils::verify_tx(&tx)?;
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }

    fn remove_mined_txs(&self, block: &Block) {
        for tx in &block.txs {
            self.pending_txs.remove(&tx.hash());
        }
    }

    // Sign a transaction on behalf of the user.
    fn sign_tx(&self, tx: Tx) -> Result<SignedTx, Error> {
        let sig = self.wallet.sign(&tx.message(), &tx.from)?;

        Ok(SignedTx {
            tx: Some(tx),
            sig: sig.into(),
        })
    }
}
