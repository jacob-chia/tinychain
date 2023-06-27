//! HTTP server that handles requests from the outside world.

use std::net::SocketAddr;

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use log::info;
use serde_json::json;

use crate::{
    biz::{Node, Peer, State},
    error::Error,
};

mod dto;

pub use dto::*;

pub async fn run<S: State, P: Peer>(addr: SocketAddr, node: Node<S, P>) {
    let router = new_router(node);

    info!("📣 HTTP server listening on {addr}");
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("Failed to run http server");
}

pub fn new_router<S: State, P: Peer>(node: Node<S, P>) -> Router {
    Router::new()
        .route("/blocks", get(get_blocks::<S, P>))
        .route("/blocks/:number", get(get_block::<S, P>))
        .route("/balances", get(get_balances::<S, P>))
        .route("/account/nonce", get(next_account_nonce::<S, P>))
        .route("/transfer", post(transfer::<S, P>))
        .fallback(not_found)
        .layer(Extension(node))
}

async fn get_blocks<S: State, P: Peer>(
    Extension(node): Extension<Node<S, P>>,
    Query(params): Query<GetBlocksReq>,
) -> impl IntoResponse {
    info!("📣 >> get_blocks by: {:?}", params);
    let blocks: Vec<BlockResp> = node
        .get_blocks(params.from_number)
        .into_iter()
        .map(BlockResp::from)
        .collect();
    info!("📣 << get_blocks response: {:?}", blocks);

    Json(blocks)
}

async fn get_block<S: State, P: Peer>(
    Extension(node): Extension<Node<S, P>>,
    Path(number): Path<u64>,
) -> impl IntoResponse {
    info!("📣 >> get_block by: {:?}", number);
    let block = node.get_block(number).map(BlockResp::from);
    info!("📣 << get_block response: {:?}", block);

    Json(block)
}

async fn get_balances<S: State, P: Peer>(
    Extension(node): Extension<Node<S, P>>,
) -> impl IntoResponse {
    info!("📣 >> get_balances");
    let resp = json!({
        "last_block_hash": node.last_block_hash(),
        "balances": node.get_balances(),
    });
    info!("📣 << get_balances response: {:?}", resp);

    Json(resp)
}

async fn next_account_nonce<S: State, P: Peer>(
    Extension(node): Extension<Node<S, P>>,
    Query(params): Query<NonceReq>,
) -> impl IntoResponse {
    info!("📣 >> next_account_nonce by: {:?}", params);
    let resp = json!({ "nonce": node.next_account_nonce(&params.account) });
    info!("📣 << next_account_nonce response: {:?}", resp);

    Json(resp)
}

async fn transfer<S: State, P: Peer>(
    Extension(node): Extension<Node<S, P>>,
    Json(tx): Json<TxReq>,
) -> Result<impl IntoResponse, HttpError> {
    info!("📣 >> transfer: {:?}", tx);
    let resp = node.transfer(&tx.from, &tx.to, tx.value, tx.nonce);
    info!("📣 << transfer response: {:?}", resp);

    resp?;
    Ok(Json(json!({"success": true})))
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

#[derive(thiserror::Error, Debug)]
enum HttpError {
    #[error("Bad request: {0}")]
    BadRequest(Error),
    #[error("Internal server error: {0}")]
    InternalServerError(Error),
}

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        match err {
            Error::BadRequest(..) => HttpError::BadRequest(err),
            _ => HttpError::InternalServerError(err),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self {
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
