use axum::Server;
use ethers_core::types::H256;
use log::{error, info};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::signal;

mod error;
mod router;

use error::*;

use crate::database::{self, Block, SignedTx, State, Tx};

// 挖矿难度
const MINING_DIFFICULTY: usize = 3;

#[derive(Debug)]
pub struct Node {
    addr: SocketAddr,
    miner: String,
    state: Box<State>,
    pending_txs: HashMap<H256, SignedTx>,
    peers: HashMap<SocketAddr, Connected>,
}

#[derive(Debug)]
struct Connected(bool);

impl Node {
    pub fn new(addr: &str, miner: &str, bootstrap_addr: &str) -> Result<Self, NodeError> {
        let state = Box::new(State::new(MINING_DIFFICULTY)?);
        let mut peers = HashMap::new();
        peers.insert(bootstrap_addr.parse()?, Connected(false));

        Ok(Self {
            addr: addr.parse()?,
            miner: miner.to_string(),
            state: state,
            pending_txs: HashMap::new(),
            peers: peers,
        })
    }

    pub async fn run(self) {
        let addr = self.addr;
        info!("Listening on {addr}");
        info!("Current state =====================================");
        info!("balances         : {:?}", self.state.get_balances());
        info!("latest_block     : {:?}", self.state.latest_block());
        info!("latest_block_hash: {:?}", self.state.latest_block_hash());

        let app = router::new_router(Arc::new(RwLock::new(self)));

        Server::bind(&addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    }

    pub fn add_tx(&mut self, from: &str, to: &str, value: u64) {
        let next_nonce = self.state.next_account_nonce(from);
        let tx = Tx::builder()
            .from(from)
            .to(to)
            .value(value)
            .nonce(next_nonce)
            .build()
            .sign();

        self.add_pending_tx(tx, self.addr);
    }

    fn add_pending_tx(&mut self, tx: SignedTx, from_peer: SocketAddr) {
        if !tx.is_valid_signature() {
            return;
        }

        info!("Added pending tx {:?} from peer {}", tx, from_peer);
        self.pending_txs.entry(tx.hash()).or_insert(tx);
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
