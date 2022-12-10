use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    error_handling::HandleErrorLayer,
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use ethers_core::types::H256;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tower::{BoxError, ServiceBuilder};

use crate::database::SignedTx;

use super::{database, Node};

type ArcNode = Arc<RwLock<Node>>;

pub fn new_router(node: ArcNode) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks))
        .route("/blocks/:number", get(get_block))
        .route("/balances", get(get_balances))
        .route("/txs", post(add_tx))
        .route("/peers", post(add_peer))
        .route("/peer/status", get(get_peer_status))
        .fallback(not_found.into_service())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_timeout_error))
                .timeout(Duration::from_secs(10))
                .layer(Extension(node))
                .into_inner(),
        )
}

async fn handle_timeout_error(err: BoxError) -> (StatusCode, String) {
    if err.is::<tower::timeout::error::Elapsed>() {
        (StatusCode::REQUEST_TIMEOUT, "Timeout".to_string())
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Unknown error: {}", err),
        )
    }
}

#[derive(Debug, Deserialize)]
struct GetBlocksReq {
    offset: usize,
}

async fn get_blocks(Query(params): Query<GetBlocksReq>) -> impl IntoResponse {
    match database::get_blocks(params.offset) {
        Ok(blocks) => Json(blocks),
        Err(err) => {
            error!("Read db error: {}", err);
            Json(vec![])
        }
    }
}

async fn get_block(Path(number): Path<u64>) -> Result<impl IntoResponse, StatusCode> {
    let block = database::get_block(number).ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(block))
}

#[derive(Debug, Serialize)]
struct BalancesResp {
    hash: H256,
    balances: HashMap<String, u64>,
}

async fn get_balances(Extension(node): Extension<ArcNode>) -> impl IntoResponse {
    let state = &node.read().unwrap().state;

    Json(BalancesResp {
        hash: state.latest_block_hash(),
        balances: state.get_balances(),
    })
}

#[derive(Debug, Deserialize)]
struct AddTxReq {
    from: String,
    to: String,
    value: u64,
}

async fn add_tx(
    Json(tx): Json<AddTxReq>,
    Extension(node): Extension<ArcNode>,
) -> impl IntoResponse {
    node.write().unwrap().add_tx(&tx.from, &tx.to, tx.value);

    (StatusCode::OK, "OK")
}

#[derive(Debug, Deserialize)]
struct AddPeerReq {
    addr: String,
}

async fn add_peer(
    Json(peer): Json<AddPeerReq>,
    Extension(node): Extension<ArcNode>,
) -> impl IntoResponse {
    node.write().unwrap().add_peer(&peer.addr);

    (StatusCode::OK, "OK")
}

#[derive(Debug, Serialize)]
struct PeerStatusResp {
    hash: H256,
    number: u64,
    peers: HashSet<SocketAddr>,
    pending_txs: Vec<SignedTx>,
}

async fn get_peer_status(Extension(node): Extension<ArcNode>) -> impl IntoResponse {
    let node = node.read().unwrap();
    Json(PeerStatusResp {
        hash: node.state.latest_block_hash(),
        number: node.state.latest_block_number(),
        peers: node.peers.clone(),
        pending_txs: node.get_pending_txs(),
    })
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
