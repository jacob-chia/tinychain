//! Miner is a background thread that periodically mines new blocks.
//!
//! When a new block is mined, it is broadcasted to other peers.
//! When a new block is received from other peers, it is added to
//! the local state and the mining process is canceled.

use std::{
    collections::HashMap,
    time::{self, Duration},
};

use crossbeam_channel::{select, tick, Receiver};
use log::{error, info};

use super::*;
use crate::{
    error::Error,
    schema::{Block, SignedTx},
    utils,
};

const MINE_INTERVAL: u64 = 30;

/// A transaction may be from users or from other peers.
/// Only the transactions from users need to be broadcasted to other peers.
#[derive(Debug)]
pub struct TxMsg {
    pub tx: SignedTx,
    pub need_broadcast: bool,
}

#[derive(Debug)]
pub struct Miner<S: State, P: PeerClient> {
    /// The pending transactions that are not yet included in a block.
    pending_txs: Vec<SignedTx>,
    /// The pending state that is used to check if a transaction is valid.
    pending_state: PendingState,
    /// The mining difficulty of the blockchain.
    mining_difficulty: usize,
    // The state of the blockchain.
    state: S,
    // The client to interact with other peers.
    peer_client: P,
    /// The address of the miner to receive mining rewards.
    author: String,
    /// The receiver of the tx from users and other peers.
    tx_receiver: Receiver<TxMsg>,
    /// The receiver of the block from other peers.
    block_receiver: Receiver<Block>,
}

/// `PendingState` merges the current `state` and the `pending_txs`.
/// It is used to check if a transaction is valid.
#[derive(Debug, Default)]
struct PendingState {
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
}

impl<S: State, P: PeerClient> Miner<S, P> {
    pub fn new(
        state: S,
        peer_client: P,
        author: String,
        mining_difficulty: usize,
        tx_receiver: Receiver<TxMsg>,
        block_receiver: Receiver<Block>,
    ) -> Self {
        let mut state = Self {
            pending_txs: Vec::new(),
            pending_state: PendingState::default(),
            mining_difficulty,
            state,
            peer_client,
            author,
            tx_receiver,
            block_receiver,
        };

        state.reset_pending_state();
        state
    }

    pub fn mine(&mut self) {
        let ticker = tick(Duration::from_secs(MINE_INTERVAL));

        loop {
            select! {
                // A new transaction is received.
                recv(self.tx_receiver) -> msg => {
                    if let Ok(tx_msg) = msg {
                        self.add_pending_tx(tx_msg);
                    }
                }
                // It's time to mine a new block.
                recv(ticker) -> _ => {
                    if self.pending_txs.is_empty() {
                        continue;
                    }

                    let block = Block::new(
                        self.state.last_block_hash().unwrap_or_default(),
                        self.state.block_height(),
                        self.author.clone(),
                        self.pending_txs.clone(),
                    );

                    if let Some(block) = self.pow(block) {
                        if self.add_block(block.clone()).is_ok() {
                            self.peer_client.broadcast_block(block);
                        }
                    }
                },
                // A new block is received.
                recv(self.block_receiver) -> msg => {
                    if let Ok(block) = msg {
                        let _ = self.add_block(block);
                    }
                }
            }
        }
    }

    fn pow(&mut self, mut block: Block) -> Option<Block> {
        let mining_difficulty = self.mining_difficulty;
        let mut attempt = 0;
        let timer = time::Instant::now();

        while !utils::is_valid_hash(&block.hash(), mining_difficulty) {
            // Every time before a new attempt, check if there are any blocks from other peers,
            // if so, cancel this mining.
            if let Ok(block) = self.block_receiver.try_recv() {
                info!("📣 Received a block from other peers, cancel mining.");
                let _ = self.add_block(block);
                return None;
            }

            if attempt % 10000 == 0 {
                let elapsed = timer.elapsed();
                info!("📣 Mining attempt: {}, elapsed: {:?}", attempt, elapsed);

                // To demonstrate that different miners have different mining power,
                // we mock a heavy work that takes random seconds.
                std::thread::sleep(Duration::from_secs(block.nonce() % 10));
            }
            attempt += 1;
            block.update_nonce_and_time();
        }

        info!("📣 Mined new Block '{}' 🎉🎉🎉:", block.hash());
        info!("📣 \tNumber: '{}'", block.number());
        info!("📣 \tNonce: '{}'", block.nonce());
        info!("📣 \tCreated: '{}'", block.timestamp());
        info!("📣 \tMiner: '{}'", block.author());
        info!("📣 \tParent: '{}'", block.parent_hash());
        info!("📣 \tAttempt: '{}'", attempt);
        info!("📣 \tTime: {:?}", timer.elapsed());
        info!("🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉\n");

        Some(block)
    }

    fn check_tx(&self, tx: &SignedTx) -> Result<(), Error> {
        utils::verify_tx_signature(tx)?;

        let from_balance = self.get_pending_balance(&tx.from);
        if from_balance < tx.cost() {
            return Err(Error::BalanceInsufficient(
                tx.from.to_string(),
                from_balance,
                tx.cost(),
            ));
        }

        let expected_nonce = self.get_pending_nonce(&tx.from);
        if expected_nonce != tx.nonce {
            return Err(Error::InvalidTxNonce(
                tx.from.to_string(),
                expected_nonce,
                tx.nonce,
            ));
        }

        let to_balance = self.get_pending_balance(&tx.to);
        if to_balance.checked_add(tx.value).is_none() {
            return Err(Error::BalanceOverflow(
                tx.to.to_string(),
                to_balance,
                tx.value,
            ));
        }

        Ok(())
    }

    fn update_pending_state(&mut self, tx: &SignedTx) {
        self.pending_state.balances.insert(
            tx.from.to_string(),
            self.get_pending_balance(&tx.from) - tx.cost(),
        );

        self.pending_state.balances.insert(
            tx.to.to_string(),
            self.get_pending_balance(&tx.to) + tx.value,
        );

        self.pending_state
            .account2nonce
            .insert(tx.from.to_string(), tx.nonce + 1);
    }

    fn reset_pending_state(&mut self) {
        self.pending_state.balances = self.state.get_balances();
        self.pending_state.account2nonce = self.state.get_account2nonce();
    }

    fn add_pending_tx(&mut self, tx_msg: TxMsg) {
        let TxMsg { tx, need_broadcast } = tx_msg;
        if let Err(err) = self.check_tx(&tx) {
            error!("❌ Bad tx: {:?}", err);
            return;
        }

        self.update_pending_state(&tx);
        self.pending_txs.push(tx.clone());
        if need_broadcast {
            self.peer_client.broadcast_tx(tx);
        }
    }

    fn check_block(&self, block: &Block) -> Result<(), Error> {
        if !utils::is_valid_hash(&block.hash(), self.mining_difficulty) {
            return Err(Error::InvalidBlockHash(
                block.hash(),
                self.mining_difficulty,
            ));
        }

        let last_block_hash = self.state.last_block_hash().unwrap_or_default();
        if last_block_hash != block.parent_hash() {
            return Err(Error::InvalidBlockParent(
                last_block_hash,
                block.parent_hash(),
            ));
        }

        let expected_number = self.state.block_height();
        if expected_number != block.number() {
            return Err(Error::InvalidBlockNumber(expected_number, block.number()));
        }

        Ok(())
    }

    fn add_block(&mut self, block: Block) -> Result<(), Error> {
        if let Err(err) = self.check_block(&block) {
            error!("❌ Bad block: {:?}", err);
            return Err(err);
        }

        let result = self.state.add_block(block);
        if result.is_ok() {
            // Once a block is added, both the pending state and pending txs are stale and should be reset.
            self.reset_pending_state();
            self.pending_txs.clear();
        }

        result
    }

    fn get_pending_balance(&self, address: &str) -> u64 {
        self.pending_state
            .balances
            .get(address)
            .cloned()
            .unwrap_or_default()
    }

    fn get_pending_nonce(&self, address: &str) -> u64 {
        self.pending_state
            .account2nonce
            .get(address)
            .cloned()
            .unwrap_or_default()
    }
}
