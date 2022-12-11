use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::{database::SignedTx, error::ChainError, types::Hash};

use super::Node;

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
        .layer(Extension(node))
}

#[derive(Debug, Serialize)]
struct OkResp {
    success: bool,
}

impl OkResp {
    pub fn new() -> Self {
        Self { success: true }
    }
}

#[derive(Debug, Deserialize)]
struct GetBlocksReq {
    offset: usize,
}

async fn get_blocks(
    Query(params): Query<GetBlocksReq>,
    Extension(node): Extension<ArcNode>,
) -> Result<impl IntoResponse, ChainError> {
    let blocks = node.read().unwrap().get_blocks(params.offset)?;
    Ok(Json(blocks))
}

async fn get_block(
    Path(number): Path<u64>,
    Extension(node): Extension<ArcNode>,
) -> Result<impl IntoResponse, ChainError> {
    let block = node.read().unwrap().get_block(number)?;
    Ok(Json(block))
}

#[derive(Debug, Serialize)]
struct BalancesResp {
    hash: Hash,
    balances: HashMap<String, u64>,
}

async fn get_balances(Extension(node): Extension<ArcNode>) -> impl IntoResponse {
    let node = node.read().unwrap();

    Json(BalancesResp {
        hash: node.latest_block_hash(),
        balances: node.get_balances(),
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
) -> Result<impl IntoResponse, ChainError> {
    node.write().unwrap().add_tx(&tx.from, &tx.to, tx.value)?;

    Ok(Json(OkResp::new()))
}

#[derive(Debug, Deserialize)]
struct AddPeerReq {
    addr: String,
}

async fn add_peer(
    Json(peer): Json<AddPeerReq>,
    Extension(node): Extension<ArcNode>,
) -> Result<impl IntoResponse, ChainError> {
    node.write().unwrap().add_peer(&peer.addr)?;

    Ok(Json(OkResp::new()))
}

#[derive(Debug, Serialize)]
struct PeerStatusResp {
    hash: Hash,
    number: u64,
    peers: HashSet<SocketAddr>,
    pending_txs: Vec<SignedTx>,
}

async fn get_peer_status(Extension(node): Extension<ArcNode>) -> impl IntoResponse {
    let node = node.read().unwrap();

    Json(PeerStatusResp {
        hash: node.latest_block_hash(),
        number: node.latest_block_number(),
        peers: node.peers.clone(),
        pending_txs: node.get_pending_txs(),
    })
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
