use std::{collections::HashMap, net::SocketAddr, time::Duration};

use crossbeam_channel::{select, tick};
use dashmap::DashMap;
use log::info;

use crate::{error::ChainError, types::Hash};

mod block;
mod miner;
mod peer;
mod state;
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

    pub state: S,
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
            state: state,
            peer_proxy: peer_proxy,
        };

        if let Some(peer) = bootstrap_addr {
            peer.parse::<SocketAddr>()?;
            node.peers.insert(peer, Connected(false));
        }

        Ok(node)
    }

    pub fn add_tx(&self, from: &str, to: &str, value: u64) -> Result<(), ChainError> {
        let next_nonce = self.state.next_account_nonce(from);
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
        self.pending_txs
            .iter()
            .map(|entry| entry.value().clone())
            .collect::<Vec<SignedTx>>()
    }

    pub fn add_peer(&self, peer: String) -> Result<(), ChainError> {
        peer.parse::<SocketAddr>()?;
        self.peers.insert(peer, Connected(true));

        Ok(())
    }

    pub fn get_peers(&self) -> Vec<String> {
        self.peers
            .iter()
            .map(|entry| entry.key().clone())
            .collect::<Vec<String>>()
    }

    pub fn get_blocks(&self, offset: usize) -> Result<Vec<Block>, ChainError> {
        self.state.get_blocks(offset)
    }

    pub fn get_block(&self, number: u64) -> Result<Block, ChainError> {
        self.state.get_block(number)
    }

    pub fn get_balances(&self) -> HashMap<String, u64> {
        self.state.get_balances()
    }

    pub fn latest_block_hash(&self) -> Hash {
        self.state.latest_block_hash()
    }

    pub fn latest_block_number(&self) -> u64 {
        self.state.latest_block_number()
    }

    pub fn mine(&self) {
        let ticker = tick(Duration::from_secs(5));

        loop {
            select! {
                recv(ticker) -> _ => {
                }
            }
        }
    }

    fn add_pending_tx(&self, tx: SignedTx, from_peer: &str) -> Result<(), ChainError> {
        tx.check_signature()?;
        info!("Added pending tx {:?} from peer {}", tx, from_peer);
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }
}
