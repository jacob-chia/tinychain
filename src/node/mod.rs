use axum::Server;
use log::info;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};
use tokio::signal;

mod error;
mod router;
mod temp;

use error::*;

use crate::database::State;

// 挖矿难度
const MINING_DIFFICULTY: usize = 3;

pub struct Node {
    addr: SocketAddr,
    miner: String,
    state: Box<State>,
    peers: HashMap<SocketAddr, Connected>,
}

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
            peers: peers,
        })
    }

    pub async fn run(self) {
        temp::temp(&self.miner);

        let addr = self.addr;
        info!("Listening on {addr}. Current state ====================");
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
