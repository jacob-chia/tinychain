use std::sync::Arc;

use axum::{
    extract::{Path, Query, State as AxumState},
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
        .route("/account/nonce", get(next_account_nonce::<S, P>))
        .route("/transfer", post(transfer::<S, P>))
        .route("/peer/ping", post(ping_peer::<S, P>))
        .route("/peer/status", get(get_peer_status::<S, P>))
        .fallback(not_found)
        .with_state(node)
}

#[derive(Debug, Deserialize)]
struct GetBlocksReq {
    offset: u64,
}

async fn get_blocks<S, P>(
    Query(params): Query<GetBlocksReq>,
    AxumState(node): AxumState<Arc<Node<S, P>>>,
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
    AxumState(node): AxumState<Arc<Node<S, P>>>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let block = node.get_block(number)?;
    Ok(Json(block))
}

async fn get_balances<S, P>(AxumState(node): AxumState<Arc<Node<S, P>>>) -> impl IntoResponse
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
struct NonceReq {
    account: String,
}

async fn next_account_nonce<S, P>(
    Query(params): Query<NonceReq>,
    AxumState(node): AxumState<Arc<Node<S, P>>>,
) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    Json(json!({ "nonce": node.next_account_nonce(&params.account) }))
}

#[derive(Debug, Deserialize)]
struct TxReq {
    from: String,
    to: String,
    value: u64,
    nonce: u64,
}

async fn transfer<S, P>(
    AxumState(node): AxumState<Arc<Node<S, P>>>,
    Json(tx): Json<TxReq>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    node.transfer(&tx.from, &tx.to, tx.value, tx.nonce)?;

    Ok(Json(json!({"success": true})))
}

#[derive(Debug, Deserialize)]
struct PingPeerReq {
    addr: String,
}

async fn ping_peer<S, P>(
    AxumState(node): AxumState<Arc<Node<S, P>>>,
    Json(peer): Json<PingPeerReq>,
) -> Result<impl IntoResponse, ChainError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    node.add_peer(peer.addr)?;

    Ok(Json(json!({"success": true})))
}

async fn get_peer_status<S, P>(AxumState(node): AxumState<Arc<Node<S, P>>>) -> impl IntoResponse
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
