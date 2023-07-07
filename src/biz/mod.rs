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

    pub fn transfer(&self, _from: &str, _to: &str, _value: u64, _nonce: u64) -> Result<(), Error> {
        todo!()
    }

    pub fn get_blocks(&self, _from_number: u64) -> Vec<Block> {
        todo!()
    }

    pub fn get_block(&self, _number: u64) -> Option<Block> {
        todo!()
    }

    pub fn get_balances(&self) -> HashMap<String, u64> {
        todo!()
    }

    pub fn block_height(&self) -> u64 {
        todo!()
    }

    pub fn last_block_hash(&self) -> Option<Hash> {
        todo!()
    }

    pub fn handle_broadcast_tx(&self, _tx: SignedTx) -> Result<(), Error> {
        todo!()
    }

    pub fn handle_broadcast_block(&self, _block: Block) {
        todo!()
    }
}
