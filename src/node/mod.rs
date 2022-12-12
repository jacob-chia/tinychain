use axum::Server;
use ethers_core::types::H256;
use log::info;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::signal;

mod router;

use crate::{database::*, error::ChainError, types::Hash};

const MINING_DIFFICULTY: usize = 3;

#[derive(Debug)]
pub struct Node {
    addr: SocketAddr,
    miner: String,
    state: Box<State>,
    pending_txs: HashMap<H256, SignedTx>,
    peers: HashSet<SocketAddr>,
}

impl Node {
    pub fn new(
        addr: String,
        miner: String,
        bootstrap_addr: Option<String>,
    ) -> Result<Self, ChainError> {
        let mut node = Self {
            addr: addr.parse()?,
            miner: miner,
            state: Box::new(State::new(MINING_DIFFICULTY)?),
            pending_txs: HashMap::new(),
            peers: HashSet::new(),
        };

        if let Some(ref bootstrap_addr) = bootstrap_addr {
            node.connect_to_peer(bootstrap_addr);
        }

        Ok(node)
    }

    pub async fn run(self) {
        let addr = self.addr;
        info!("Listening on {addr}");
        info!("Current state =====================================");
        info!("balances         : {:?}", self.get_balances());
        info!("latest_block     : {:?}", self.state.latest_block());
        info!("latest_block_hash: {:?}", self.latest_block_hash());

        let app = router::new_router(Arc::new(RwLock::new(self)));

        Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
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

    fn connect_to_peer(&mut self, peer: &str) {
        if peer == self.addr.to_string() {
            return;
        }

        // if add peer ok
        self.add_peer(peer);
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
