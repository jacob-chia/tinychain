use std::sync::Arc;

use axum::{
    extract::{Extension, Path, Query},
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::ChainError,
    node::{Node, Peer, State},
};

pub fn new_router<S, P>(node: Arc<Node<S, P>>) -> Router
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

#[derive(Debug, Deserialize)]
struct GetBlocksReq {
    offset: usize,
}

async fn get_blocks<S, P>(
    Query(params): Query<GetBlocksReq>,
    Extension(node): Extension<Arc<Node<S, P>>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let blocks = node.get_blocks(params.offset)?;
    Ok(Json(blocks))
}

async fn get_block<S, P>(
    Path(number): Path<u64>,
    Extension(node): Extension<Arc<Node<S, P>>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let block = node.get_block(number)?;
    Ok(Json(block))
}

async fn get_balances<S, P>(Extension(node): Extension<Arc<Node<S, P>>>) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    Json(json!({
        "hash": node.latest_block_hash(),
        "balances": node.get_balances(),
    }))
}

#[derive(Debug, Deserialize)]
struct AddTxReq {
    from: String,
    to: String,
    value: u64,
}

async fn add_tx<S, P>(
    Json(tx): Json<AddTxReq>,
    Extension(node): Extension<Arc<Node<S, P>>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    node.add_tx(&tx.from, &tx.to, tx.value)?;

    Ok(Json(json!({"success": true})))
}

#[derive(Debug, Deserialize)]
struct PingPeerReq {
    addr: String,
}

async fn ping_peer<S, P>(
    Json(peer): Json<PingPeerReq>,
    Extension(node): Extension<Arc<Node<S, P>>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    node.add_peer(peer.addr)?;

    Ok(Json(json!({"success": true})))
}

async fn get_peer_status<S, P>(Extension(node): Extension<Arc<Node<S, P>>>) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    Json(json!( {
        "hash": node.latest_block_hash(),
        "number": node.latest_block_number(),
        "peers": node.get_peers(),
        "pending_txs": node.get_pending_txs(),
    }))
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
