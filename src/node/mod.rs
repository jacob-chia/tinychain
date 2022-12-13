use log::info;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
};

use crate::{error::ChainError, types::Hash};

mod block;
mod peer;
mod state;
mod tx;

pub use block::*;
pub use peer::*;
pub use state::*;
pub use tx::*;

#[derive(Debug)]
pub struct Node<S, P> {
    pub(crate) addr: SocketAddr,
    pub(crate) miner: String,
    pub(crate) pending_txs: HashMap<Hash, SignedTx>,
    pub(crate) peers: HashSet<SocketAddr>,
    pub(crate) state: Box<S>,
    pub(crate) peer_proxy: Box<P>,
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
        let mut node = Self {
            addr: addr.parse()?,
            miner: miner,
            pending_txs: HashMap::new(),
            peers: HashSet::new(),
            state: Box::new(state),
            peer_proxy: Box::new(peer_proxy),
        };

        if let Some(ref bootstrap_addr) = bootstrap_addr {
            node.connect_to_peer(bootstrap_addr).await?;
        }

        Ok(node)
    }

    pub fn add_tx(&mut self, from: &str, to: &str, value: u64) -> Result<(), ChainError> {
        let next_nonce = self.state.next_account_nonce(from);
        let tx = Tx::builder()
            .from(from)
            .to(to)
            .value(value)
            .nonce(next_nonce)
            .build()
            .sign()?;

        self.add_pending_tx(tx, self.addr)
    }

    pub fn get_pending_txs(&self) -> Vec<SignedTx> {
        self.pending_txs
            .iter()
            .map(|(_, tx)| tx.clone())
            .collect::<Vec<SignedTx>>()
    }

    pub fn add_peer(&mut self, peer: &str) -> Result<(), ChainError> {
        self.peers.insert(peer.parse()?);

        Ok(())
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

    fn add_pending_tx(&mut self, tx: SignedTx, from_peer: SocketAddr) -> Result<(), ChainError> {
        tx.check_signature()?;
        info!("Added pending tx {:?} from peer {}", tx, from_peer);
        self.pending_txs.entry(tx.hash()).or_insert(tx);

        Ok(())
    }

    async fn connect_to_peer(&mut self, peer: &str) -> Result<(), ChainError> {
        if peer == self.addr.to_string() {
            return Ok(());
        }

        self.peer_proxy.ping(&self.addr.to_string(), peer).await?;
        self.peers.insert(peer.parse()?);

        Ok(())
    }
}
