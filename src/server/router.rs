use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use crate::{
    error::ChainError,
    node::{Node, Peer, SignedTx, State},
    types::Hash,
};

type ArcNode<S, P> = Arc<RwLock<Node<S, P>>>;

pub fn new_router<S, P>(node: ArcNode<S, P>) -> Router
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    Router::new()
        .route("/blocks", get(get_blocks::<S, P>))
        .route("/blocks/:number", get(get_block::<S, P>))
        .route("/balances", get(get_balances::<S, P>))
        .route("/txs", post(add_tx::<S, P>))
        .route("/peer/ping", post(ping_peer::<S, P>))
        .route("/peer/status", get(get_peer_status::<S, P>))
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

async fn get_blocks<S, P>(
    Query(params): Query<GetBlocksReq>,
    Extension(node): Extension<ArcNode<S, P>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let blocks = node.read().unwrap().get_blocks(params.offset)?;
    Ok(Json(blocks))
}

async fn get_block<S, P>(
    Path(number): Path<u64>,
    Extension(node): Extension<ArcNode<S, P>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let block = node.read().unwrap().get_block(number)?;
    Ok(Json(block))
}

#[derive(Debug, Serialize)]
struct BalancesResp {
    hash: Hash,
    balances: HashMap<String, u64>,
}

async fn get_balances<S, P>(Extension(node): Extension<ArcNode<S, P>>) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
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

async fn add_tx<S, P>(
    Json(tx): Json<AddTxReq>,
    Extension(node): Extension<ArcNode<S, P>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    node.write().unwrap().add_tx(&tx.from, &tx.to, tx.value)?;

    Ok(Json(OkResp::new()))
}

#[derive(Debug, Deserialize)]
struct PingPeerReq {
    addr: String,
}

async fn ping_peer<S, P>(
    Json(peer): Json<PingPeerReq>,
    Extension(node): Extension<ArcNode<S, P>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
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

async fn get_peer_status<S, P>(Extension(node): Extension<ArcNode<S, P>>) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
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
