//! The entry point of the blockchain node.

use std::{collections::HashMap, marker};

mod peer_client;
mod state;

pub use peer_client::*;
pub use state::*;

use crate::{
    error::Error,
    schema::{Block, SignedTx},
    types::Hash,
};

#[derive(Debug, Clone)]
pub struct Node<S: State, P: PeerClient> {
    _marker: marker::PhantomData<(S, P)>,
}

impl<S: State, P: PeerClient> Node<S, P> {
    pub fn new(_p2p_client: P) -> Self {
        Self {
            _marker: marker::PhantomData,
        }
    }

    /// Get the next nouce of the given account.
    /// The nounce is a monotonically increasing number that is used to prevent replay attacks.
    pub fn next_account_nonce(&self, _account: &str) -> u64 {
        todo!()
    }

    /// Transfer the `value` from `from` to `to`.
    pub fn transfer(&self, _from: &str, _to: &str, _value: u64, _nonce: u64) -> Result<(), Error> {
        todo!()
    }

    /// Get blocks from the given number.
    pub fn get_blocks(&self, _from_number: u64) -> Vec<Block> {
        todo!()
    }

    /// Get the block by the given number.
    pub fn get_block(&self, _number: u64) -> Option<Block> {
        todo!()
    }

    /// Get all the balances of the accounts.
    pub fn get_balances(&self) -> HashMap<String, u64> {
        todo!()
    }

    /// Get the block height.
    pub fn block_height(&self) -> u64 {
        todo!()
    }

    /// Get the last block hash.
    pub fn last_block_hash(&self) -> Option<Hash> {
        todo!()
    }

    /// Add a pending transaction to the transaction pool.
    pub fn add_pending_tx(&self, _tx: SignedTx) -> Result<(), Error> {
        todo!()
    }

    /// Add a block and stop the current mining process.
    pub fn add_block_stop_mining(&self, _block: Block) {
        todo!()
    }
}
