//! HTTP server that handles requests from the outside world.

use std::net::SocketAddr;

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router, Server,
};
use log::info;

use crate::biz::{Node, PeerClient, State};

mod dto;

pub use dto::*;

pub async fn run<S: State, P: PeerClient>(addr: SocketAddr, node: Node<S, P>) {
    let router = new_router(node);

    info!("📣 HTTP server listening on {addr}");
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("Failed to run http server");
}

pub fn new_router<S: State, P: PeerClient>(node: Node<S, P>) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks::<S, P>))
        .route("/blocks/:number", get(get_block::<S, P>))
        .route("/balances", get(get_balances::<S, P>))
        .route("/account/nonce", get(next_account_nonce::<S, P>))
        .route("/transfer", post(transfer::<S, P>))
        .fallback(not_found)
        .layer(Extension(node))
}

async fn get_blocks<S: State, P: PeerClient>(
    Extension(_node): Extension<Node<S, P>>,
    Query(_params): Query<GetBlocksReq>,
) -> impl IntoResponse {
    todo!()
}

async fn get_block<S: State, P: PeerClient>(
    Extension(_node): Extension<Node<S, P>>,
    Path(_number): Path<u64>,
) -> impl IntoResponse {
    todo!()
}

async fn get_balances<S: State, P: PeerClient>(
    Extension(_node): Extension<Node<S, P>>,
) -> impl IntoResponse {
    todo!()
}

async fn next_account_nonce<S: State, P: PeerClient>(
    Extension(_node): Extension<Node<S, P>>,
    Query(_params): Query<NonceReq>,
) -> impl IntoResponse {
    todo!()
}

async fn transfer<S: State, P: PeerClient>(
    Extension(_node): Extension<Node<S, P>>,
    Json(_tx): Json<TxReq>,
) -> impl IntoResponse {
    todo!()
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
