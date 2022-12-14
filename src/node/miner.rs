use std::time::{self, Duration};

use crossbeam_channel::{select, tick, Receiver};
use log::error;

use super::*;
use crate::utils;

const MINE_INTERVAL: u64 = 5;

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn mine(&self, block_receiver: Receiver<Block>) {
        info!("Miner is running ====================");
        let ticker = tick(Duration::from_secs(MINE_INTERVAL));

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
                // 收到来自其他节点的区块，此时尚未开始挖矿
                recv(block_receiver) -> block => {
                    if let Ok(block) = block {
                        info!("Received a block (hash: {}) from another peer.",block.hash());
                        self.add_block(block);
                    }
                }
            }
        }
    }

    fn pow(&self, mut block: Block, block_receiver: Receiver<Block>) -> Option<Block> {
        let mining_difficulty = self.mining_difficulty;
        // 尝试次数
        let mut attempt = 0;
        let timer = time::Instant::now();

        while !utils::is_valid_hash(&block.hash(), mining_difficulty) {
            // 每次新的尝试之前，先检查有没有同步到来自其他peers的区块
            // 若收到新的区块，取消本次挖矿
            if let Ok(block) = block_receiver.try_recv() {
                info!(
                    "Mining cancelled. Received a block (hash: {}) from another peer.",
                    block.hash()
                );
                self.add_block(block);
                return None;
            }

            if attempt % 1000000 == 0 {
                info!("Mining attempt: {attempt}, elapsed: {:?}", timer.elapsed());
            }
            attempt += 1;
            block.update_nonce(utils::gen_random_number());
        }

        info!("Mined new Block '{}' using PoW🎉🎉🎉:", block.hash());
        info!("\tHeight: '{}'", block.header.number);
        info!("\tNonce: '{}'", block.header.nonce);
        info!("\tCreated: '{}'", block.header.time);
        info!("\tMiner: '{}'", block.header.miner);
        info!("\tParent: '{}'\n", block.header.parent);
        info!("\tAttempt: '{}'", attempt);
        info!("\tTime: {:?}", timer.elapsed());
        info!("🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉🎉\n");

        Some(block)
    }

    fn add_block(&self, block: Block) {
        self.remove_mined_txs(&block);
        if let Err(err) = self.state.write().unwrap().add_block(block) {
            error!("Failed to add block: {}", err);
        }
    }
}
