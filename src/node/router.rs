use std::sync::{Arc, RwLock};

use axum::{
    handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};

use super::{error::*, Node};

pub fn new_router(node: Arc<RwLock<Node>>) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks))
        .route("/blocks/:number_or_hash", get(get_block))
        .route("/balances", get(get_balances))
        .route("/txs", post(add_tx))
        .route("/peers", post(add_peer))
        .route("/node/status", get(get_node_status))
        .fallback(not_found.into_service())
}

async fn get_blocks() -> Result<(), NodeError> {
    Ok(())
}

async fn get_block() -> Result<(), NodeError> {
    Ok(())
}

async fn get_balances() -> Result<(), NodeError> {
    Ok(())
}

async fn add_tx() -> Result<(), NodeError> {
    Ok(())
}

async fn add_peer() -> Result<(), NodeError> {
    Ok(())
}

async fn get_node_status() -> Result<(), NodeError> {
    Ok(())
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
