use std::time::{self, Duration};

use crossbeam_channel::{select, tick, Receiver};
use log::error;

use super::*;
use crate::utils;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn mine(&self, block_receiver: Receiver<Block>) {
        let ticker = tick(Duration::from_secs(5));

        loop {
            select! {
                recv(ticker) -> _ => {
                    ticker.recv().unwrap();
                    if self.pending_txs.is_empty() {
                        continue;
                    }

                    let block = Block::builder()
                        .parent(self.latest_block_hash())
                        .number(self.next_block_number())
                        .time(utils::unix_timestamp())
                        .nonce(utils::gen_random_number())
                        .miner(&self.miner)
                        .txs(self.get_pending_txs())
                        .build();

                    if let Some(block) = self.pow(block, block_receiver.clone()) {
                        self.add_block(block);
                    }
                },
                recv(block_receiver) -> block => {
                    if let Ok(block) = block {
                        self.add_block(block);
                    }
                }
            }
        }
    }

    fn pow(&self, mut block: Block, block_receiver: Receiver<Block>) -> Option<Block> {
        let mining_difficulty = self.mining_difficulty;
        let mut attempt = 0;
        let timer = time::Instant::now();

        while !utils::is_valid_hash(&block.hash(), mining_difficulty) {
            if let Ok(block) = block_receiver.try_recv() {
                self.add_block(block);
                info!("Mining cancelled. Received a block from another peer.");
                return None;
            }

            if attempt % 1000000 == 0 {
                info!(
                    "Mining attempt: {}, elapsed: {:?}",
                    attempt,
                    timer.elapsed()
                );
            }

            attempt += 1;
            block.update_nonce(utils::gen_random_number());
        }

        info!("Mined new Block '{}' using PoW🎉🎉🎉:\n", block.hash());
        info!("\tHeight: '{}'", block.header.number);
        info!("\tNonce: '{}'", block.header.nonce);
        info!("\tCreated: '{}'", block.header.time);
        info!("\tMiner: '{}'", block.header.miner);
        info!("\tParent: '{}'", block.header.parent);
        info!("\tAttempt: '{}'", attempt);
        info!("\tTime: {:?}", timer.elapsed());

        Some(block)
    }

    fn add_block(&self, block: Block) {
        self.remove_mined_txs(&block);
        if let Err(err) = self.state.write().unwrap().add_block(block) {
            error!("Failed to add block: {}", err);
        }
    }
}
