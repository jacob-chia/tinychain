use std::{collections::HashMap, net::SocketAddr};

use dashmap::{DashMap, DashSet};
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
    pub peers: DashSet<String>,
    pub pending_txs: DashMap<Hash, SignedTx>,

    pub state: S,
    pub peer_proxy: P,
}

impl<S, P> Node<S, P>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    pub async fn new(
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
            peers: DashSet::new(),
            pending_txs: DashMap::new(),
            state: state,
            peer_proxy: peer_proxy,
        };

        if let Some(bootstrap_addr) = bootstrap_addr {
            node.connect_to_peer(bootstrap_addr).await?;
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
        self.peers.insert(peer);

        Ok(())
    }

    pub fn get_peers(&self) -> Vec<String> {
        Vec::from_iter(self.peers.clone())
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

    fn add_pending_tx(&self, tx: SignedTx, from_peer: &str) -> Result<(), ChainError> {
        tx.check_signature()?;
        info!("Added pending tx {:?} from peer {}", tx, from_peer);
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }

    async fn connect_to_peer(&self, peer: String) -> Result<(), ChainError> {
        if peer == self.addr {
            return Ok(());
        }

        self.peer_proxy.ping(&self.addr, &peer).await?;
        self.peers.insert(peer);

        Ok(())
    }
}
