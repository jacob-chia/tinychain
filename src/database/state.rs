use anyhow::{anyhow, Result};
use ethers_core::types::H256;
use log::info;
use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
};

use super::*;

#[derive(Debug, Default, Clone)]
pub struct State {
    balances: HashMap<String, u64>,
    account2nonce: HashMap<String, u64>,
    latest_block: Block,
    latest_block_hash: H256,
    mining_difficulty: usize,
    has_blocks: bool,
}

impl State {
    pub fn new(mining_difficulty: usize) -> Result<Self> {
        let genesis = Genesis::load()?;
        info!("Genesis loaded, token symbol: {}", genesis.symbol);

        let mut state = Self {
            balances: genesis.clone_balances(),
            mining_difficulty: mining_difficulty,
            ..Default::default()
        };

        state.load_db()?;
        Ok(state)
    }

    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.balances.clone()
    }

    pub fn next_block_number(&self) -> u64 {
        if self.has_blocks {
            return self.latest_block.header.number + 1;
        }

        0
    }

    pub fn next_account_nonce(&self, account: &str) -> u64 {
        *self.account2nonce.get(account).unwrap_or(&0) + 1
    }

    pub fn latest_block(&self) -> &Block {
        &self.latest_block
    }

    pub fn latest_block_hash(&self) -> &H256 {
        &self.latest_block_hash
    }

    pub fn add_block(&mut self, block: Block) -> Result<H256> {
        // 为什么需要clone？
        // 当中间过程出错时，state需要回滚状态
        // 一个简单的实现是：现在clone上执行，当没有错误时再用clone替换本体
        let mut state = self.clone();
        state.apply_block(block)?;
        state.persist()?;
        *self = state;

        Ok(self.latest_block_hash)
    }

    fn load_db(&mut self) -> Result<()> {
        let db_path = BLOCKDB_PATH.get().unwrap();
        let db = File::open(db_path)?;
        let lines = BufReader::new(db).lines();

        for line in lines {
            if let Ok(ref block_str) = line {
                let mut block_kv: BlockKV = serde_json::from_str(block_str)?;
                let block = block_kv.take_block();
                self.apply_block(block)?;
            }
        }

        Ok(())
    }

    fn persist(&self) -> Result<()> {
        let mut block_json = serde_json::to_string(&BlockKV {
            key: self.latest_block_hash,
            value: self.latest_block.clone(),
        })?;
        block_json.push('\n');

        let db_path = BLOCKDB_PATH.get().unwrap();
        OpenOptions::new()
            .append(true)
            .open(db_path)?
            .write_all(block_json.as_bytes())?;

        Ok(())
    }

    fn apply_block(&mut self, block: Block) -> Result<()> {
        self.check_block(&block)?;
        self.apply_txs(&block.txs)?;

        *self
            .balances
            .entry(block.header.miner.to_owned())
            .or_default() += block.block_reward();

        self.latest_block_hash = block.hash();
        self.latest_block = block;
        self.has_blocks = true;

        Ok(())
    }

    fn check_block(&self, block: &Block) -> Result<()> {
        // check number
        let expected_number = self.latest_block.header.number + 1;
        if self.has_blocks && expected_number != block.header.number {
            return Err(anyhow!(
                "block number error: expected '{}', not '{}'",
                expected_number,
                block.header.number
            ));
        }

        // check header
        if self.has_blocks && self.latest_block_hash != block.header.parent {
            return Err(anyhow!(
                "block parent error: expected '{}', not '{}'",
                self.latest_block_hash,
                block.header.parent
            ));
        }

        // check hash
        if !self.is_valid_hash(&block.hash()) {
            return Err(anyhow!("invalid block hash: '{}'", block.hash()));
        }

        Ok(())
    }

    fn is_valid_hash(&self, hash: &H256) -> bool {
        let hash_prefix = vec![0u8; self.mining_difficulty];
        // TODO
        hash_prefix[..] != hash[..self.mining_difficulty]
    }

    fn apply_txs(&mut self, signed_txs: &[SignedTx]) -> Result<()> {
        for signed_tx in signed_txs {
            self.check_tx(signed_tx)?;

            let tx = &signed_tx.tx;
            *self.balances.get_mut(&tx.from).unwrap() -= tx.cost();
            *self.balances.entry(tx.to.to_owned()).or_default() += tx.value;
            *self.account2nonce.entry(tx.from.to_owned()).or_default() = tx.nonce;
        }

        Ok(())
    }

    fn check_tx(&self, signed_tx: &SignedTx) -> Result<()> {
        // check signature
        if !signed_tx.is_valid_signature() {
            return Err(anyhow!("wrong Tx. invalid signature"));
        }

        let tx = &signed_tx.tx;

        // check account nonce
        let expected_nonce = self.next_account_nonce(&tx.from);
        if expected_nonce != tx.nonce {
            return Err(anyhow!(
                "wrong Tx. Sender '{}' nonce error: expected '{}', not '{}'",
                tx.from,
                expected_nonce,
                tx.nonce
            ));
        }

        // check balance
        let balance = *self.balances.get(&tx.from).unwrap_or(&0);
        if balance < tx.cost() {
            return Err(anyhow!(
                "wrong Tx. Sender '{}' insufficient balance: cost '{}', balance is '{}'",
                tx.from,
                tx.cost(),
                balance
            ));
        }

        Ok(())
    }
}
