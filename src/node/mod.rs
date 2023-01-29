use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use log::info;

use crate::{error::ChainError, types::Hash};

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
    pub addr: String,
    pub miner: String,
    pub peers: DashMap<String, Connected>,
    // 尚未生成区块的TXs
    pub pending_txs: DashMap<Hash, SignedTx>,
    pub mining_difficulty: usize,

    pub state: Arc<RwLock<S>>,
    // Peer的代理，通过 peer_proxy 获取其他 peers 的数据
    pub peer_proxy: P,
}

#[derive(Debug, Clone)]
pub struct Connected(bool);

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub fn new(
        addr: String,
        miner: String,
        bootstrap_addr: Option<String>,
        state: S,
        peer_proxy: P,
    ) -> Result<Self, ChainError> {
        addr.parse::<SocketAddr>()?;

        let node = Self {
            addr,
            miner,
            peers: DashMap::new(),
            pending_txs: DashMap::new(),
            mining_difficulty: state.get_mining_difficulty(),
            state: Arc::new(RwLock::new(state)),
            peer_proxy,
        };

        if let Some(peer) = bootstrap_addr {
            peer.parse::<SocketAddr>()?;
            if peer != node.addr {
                node.peers.insert(peer, Connected(false));
            }
        }

        Ok(node)
    }

    pub fn next_account_nonce(&self, account: &str) -> u64 {
        self.state.read().unwrap().next_account_nonce(account)
    }

    /// 发送一笔交易（Tx）。
    pub fn transfer(&self, from: &str, to: &str, value: u64, nonce: u64) -> Result<(), ChainError> {
        let tx = Tx::builder()
            .from(from)
            .to(to)
            .value(value)
            .nonce(nonce)
            .build()
            .sign()?;

        self.add_pending_tx(tx, &self.addr)
    }

    /// 获取未生成区块的Tx，注意要按交易时间排序。
    pub fn get_pending_txs(&self) -> Vec<SignedTx> {
        let mut txs = self
            .pending_txs
            .iter()
            .map(|entry| entry.value().to_owned())
            .collect::<Vec<SignedTx>>();

        txs.sort_by_key(|tx| tx.time());
        txs
    }

    /// 添加 peer。当收到来自其他 peer 的 ping 时执行此操作。
    pub fn add_peer(&self, peer: String) -> Result<(), ChainError> {
        peer.parse::<SocketAddr>()?;
        if peer != self.addr {
            info!("Connected to {peer}");
            self.peers.insert(peer, Connected(true));
        }

        Ok(())
    }

    /// 获取本节点知道的 peers。
    pub fn get_peers(&self) -> Vec<String> {
        self.peers
            .iter()
            .map(|entry| entry.key().to_owned())
            .collect::<Vec<String>>()
    }

    /// 获取从 offset (即number) 处开始的所有区块。
    pub fn get_blocks(&self, offset: u64) -> Result<Vec<Block>, ChainError> {
        self.state.read().unwrap().get_blocks(offset)
    }

    /// 获取指定 number 的区块
    pub fn get_block(&self, number: u64) -> Result<Block, ChainError> {
        self.state.read().unwrap().get_block(number)
    }

    /// 获取所有人的余额
    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.read().unwrap().get_balances()
    }

    /// 获取最新的区块 hash
    pub fn latest_block_hash(&self) -> Hash {
        self.state.read().unwrap().latest_block_hash()
    }

    /// 获取最新的区块 number
    pub fn latest_block_number(&self) -> u64 {
        self.state.read().unwrap().latest_block_number()
    }

    /// 获取下一个区块 number
    pub fn next_block_number(&self) -> u64 {
        self.state.read().unwrap().next_block_number()
    }

    fn add_pending_tx(&self, tx: SignedTx, peer_addr: &str) -> Result<(), ChainError> {
        tx.check_signature()?;
        info!("Added pending tx {:?} from peer {}", tx, peer_addr);
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }

    fn remove_mined_txs(&self, block: &Block) {
        for tx in &block.txs {
            self.pending_txs.remove(&tx.hash());
        }
    }

    fn remove_peer(&self, peer: &str) {
        self.peers.remove(peer);
    }
}
