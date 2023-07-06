use std::collections::HashMap;

use crossbeam_channel::Sender;
use wallet::Wallet;

use crate::{error::Error, schema::*, types::Hash};

use super::{miner::TxMsg, State};

#[derive(Debug, Clone)]
pub struct Node<S: State> {
    // A state machine that holds the state of the blockchain.
    state: S,
    // A channel to send a signed transaction to the miner.
    tx_sender: Sender<TxMsg>,
    // A channel to send a block to the miner.
    block_sender: Sender<Block>,

    // For facilitating a smooth demonstration, the node holds a wallet that stores all
    // the keys of the users, so that it can sign transactions on behalf of the users.
    // In the real world, every user should have their own wallet.
    wallet: Wallet,
}

impl<S: State> Node<S> {
    pub fn new(
        state: S,
        wallet: Wallet,
        tx_sender: Sender<TxMsg>,
        block_sender: Sender<Block>,
    ) -> Self {
        Self {
            state,
            tx_sender,
            block_sender,
            wallet,
        }
    }

    /// Get the next nouce of the given account.
    /// The nounce is a monotonically increasing number that is used to prevent replay attacks.
    pub fn next_account_nonce(&self, account: &str) -> u64 {
        self.state.next_account_nonce(account)
    }

    pub fn transfer(&self, from: &str, to: &str, value: u64, nonce: u64) -> Result<(), Error> {
        let tx = Tx::new(from, to, value, nonce);
        let signed_tx = self.sign_tx(tx)?;
        let _ = self.tx_sender.send(TxMsg {
            tx: signed_tx,
            need_broadcast: true,
        });

        Ok(())
    }

    pub fn get_blocks(&self, from_number: u64) -> Vec<Block> {
        self.state.get_blocks(from_number)
    }

    pub fn get_block(&self, number: u64) -> Option<Block> {
        self.state.get_block(number)
    }

    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.get_balances()
    }

    pub fn block_height(&self) -> u64 {
        self.state.block_height()
    }

    pub fn last_block_hash(&self) -> Option<Hash> {
        self.state.last_block_hash()
    }

    pub fn handle_broadcast_block(&self, block: Block) {
        let _ = self.block_sender.send(block);
    }

    pub fn handle_broadcast_tx(&self, tx: SignedTx) {
        let _ = self.tx_sender.send(TxMsg {
            tx,
            need_broadcast: false,
        });
    }

    // Sign a transaction on behalf of users.
    fn sign_tx(&self, tx: Tx) -> Result<SignedTx, Error> {
        let sig = self.wallet.sign(&tx.as_bytes(), &tx.from)?;

        Ok(SignedTx {
            tx: Some(tx),
            sig: sig.to_vec(),
        })
    }
}
