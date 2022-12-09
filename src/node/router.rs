use std::{
    collections::HashMap,
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

use super::{database, error::*, Node};

type ArcNode = Arc<RwLock<Node>>;

pub fn new_router(node: ArcNode) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks))
        .route("/blocks/:number", get(get_block))
        .route("/balances", get(get_balances))
        .route("/txs", post(add_tx))
        .route("/peers", post(add_peer))
        .route("/node/status", get(get_node_status))
        .fallback(not_found.into_service())
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(handle_error_layer))
                .timeout(Duration::from_secs(10))
                .layer(Extension(node))
                .into_inner(),
        )
}

async fn handle_error_layer(err: BoxError) -> (StatusCode, String) {
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

async fn add_tx() -> impl IntoResponse {
    todo!()
}

async fn add_peer() -> impl IntoResponse {
    todo!()
}

async fn get_node_status() -> impl IntoResponse {
    todo!()
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
