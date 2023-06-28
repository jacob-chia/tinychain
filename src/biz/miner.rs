//! Periodically mine blocks.
//!
//! Cancel mining if received a block from other peers when mining.

use std::time::{self, Duration};

use crossbeam_channel::{select, tick, Receiver};
use log::info;

use super::*;
use crate::utils;

const MINE_INTERVAL: u64 = 11;

impl<S: State, P: PeerClient> NodeInner<S, P> {
    pub fn mine(&self, cancel_signal_r: Receiver<()>) {
        let ticker = tick(Duration::from_secs(MINE_INTERVAL));

        loop {
            select! {
                // It's time to mine a new block.
                recv(ticker) -> _ => {
                    if self.pending_txs.is_empty() {
                        continue;
                    }

                    let block = Block::new(
                        self.last_block_hash().unwrap_or_default(),
                        self.block_height(),
                        self.author.clone(),
                        self.get_pending_txs(),
                    );

                    if let Some(block) = self.pow(block, cancel_signal_r.clone()) {
                        if self.add_block(block.clone()).is_ok() {
                            self.peer_client.broadcast_block(block)
                        }
                    }
                },
                // Miner is not mining right now, ignore the cancel signal.
                recv(cancel_signal_r) -> _ => {
                    continue;
                }
            }
        }
    }

    fn get_pending_txs(&self) -> Vec<SignedTx> {
        let mut txs = self
            .pending_txs
            .iter()
            .map(|entry| entry.value().to_owned())
            .collect::<Vec<SignedTx>>();

        txs.sort_by_key(|tx| tx.timestamp);
        txs
    }

    fn pow(&self, mut block: Block, cancel_signal_r: Receiver<()>) -> Option<Block> {
        let mining_difficulty = self.mining_difficulty;
        let mut attempt = 0;
        let timer = time::Instant::now();

        while !utils::is_valid_hash(&block.hash(), mining_difficulty) {
            // Every time before a new attempt, check if there are any blocks from other peers,
            // if so, cancel this mining.
            if cancel_signal_r.try_recv().is_ok() {
                info!("📣 Received block from other peers, cancel mining.");
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
}
