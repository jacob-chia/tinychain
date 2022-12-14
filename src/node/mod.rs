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
    pub pending_txs: DashMap<Hash, SignedTx>,
    pub mining_difficulty: usize,

    pub state: Arc<RwLock<S>>,
    pub peer_proxy: P,
}

#[derive(Debug)]
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
            addr: addr,
            miner: miner,
            peers: DashMap::new(),
            pending_txs: DashMap::new(),
            mining_difficulty: state.get_mining_difficulty(),
            state: Arc::new(RwLock::new(state)),
            peer_proxy: peer_proxy,
        };

        if let Some(peer) = bootstrap_addr {
            peer.parse::<SocketAddr>()?;
            if peer != node.addr {
                node.peers.insert(peer, Connected(false));
            }
        }

        Ok(node)
    }

    pub fn transfer(&self, from: &str, to: &str, value: u64) -> Result<(), ChainError> {
        let next_nonce = self.state.read().unwrap().next_account_nonce(from);
        let tx = Tx::builder()
            .from(from)
            .to(to)
            .value(value)
            .nonce(next_nonce)
            .build()
            .sign()?;

        self.add_pending_tx(tx, &self.addr)
    }

    pub fn get_pending_txs(&self) -> Vec<SignedTx> {
        let mut txs = self
            .pending_txs
            .iter()
            .map(|entry| entry.value().clone())
            .collect::<Vec<SignedTx>>();

        txs.sort_by(|tx1, tx2| tx1.time().cmp(&tx2.time()));
        txs
    }

    pub fn add_peer(&self, peer: String) -> Result<(), ChainError> {
        peer.parse::<SocketAddr>()?;
        if peer != self.addr {
            self.peers.insert(peer, Connected(true));
        }

        Ok(())
    }

    pub fn get_peers(&self) -> Vec<String> {
        self.peers
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<String>>()
    }

    pub fn get_blocks(&self, offset: u64) -> Result<Vec<Block>, ChainError> {
        self.state.read().unwrap().get_blocks(offset)
    }

    pub fn get_block(&self, number: u64) -> Result<Block, ChainError> {
        self.state.read().unwrap().get_block(number)
    }

    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.read().unwrap().get_balances()
    }

    pub fn latest_block_hash(&self) -> Hash {
        self.state.read().unwrap().latest_block_hash()
    }

    pub fn latest_block_number(&self) -> u64 {
        self.state.read().unwrap().latest_block_number()
    }

    pub fn next_block_number(&self) -> u64 {
        self.state.read().unwrap().next_block_number()
    }

    fn add_pending_tx(&self, tx: SignedTx, from_peer: &str) -> Result<(), ChainError> {
        tx.check_signature()?;
        info!("Added pending tx {:?} from peer {}", tx, from_peer);
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
