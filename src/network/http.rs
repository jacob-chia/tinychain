//! HTTP server that handles requests from the outside world.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router, Server,
};
use log::info;
use serde::Deserialize;
use serde_json::json;

use crate::{
    error::Error,
    node::{Node, Peer, State},
};

pub async fn run<S, P>(addr: SocketAddr, node: Arc<Node<S, P>>)
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    let router = new_router(node);

    info!("📣 HTTP server listening on {addr}");
    Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .expect("Failed to run http server");
}

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
        .fallback(not_found)
        .layer(Extension(node))
}

#[derive(Debug, Deserialize)]
struct GetBlocksReq {
    from_number: u64,
}

async fn get_blocks<S, P>(
    Extension(node): Extension<Arc<Node<S, P>>>,
    Query(params): Query<GetBlocksReq>,
) -> Result<impl IntoResponse, HttpError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    info!("📣 >> get_blocks by: {:?}", params);
    let blocks = node.get_blocks(params.from_number)?;
    info!("📣 << get_blocks response: {:?}", blocks);

    Ok(Json(blocks))
}

async fn get_block<S, P>(
    Extension(node): Extension<Arc<Node<S, P>>>,
    Path(number): Path<u64>,
) -> Result<impl IntoResponse, HttpError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    info!("📣 >> get_block by: {:?}", number);
    let block = node.get_block(number)?;
    info!("📣 << get_block response: {:?}", block);

    Ok(Json(block))
}

async fn get_balances<S, P>(Extension(node): Extension<Arc<Node<S, P>>>) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    info!("📣 >> get_balances");
    let resp = json!({
        "hash": node.latest_block_hash(),
        "balances": node.get_balances(),
    });
    info!("📣 << get_balances response: {:?}", resp);

    Json(resp)
}

#[derive(Debug, Deserialize)]
struct NonceReq {
    account: String,
}

async fn next_account_nonce<S, P>(
    Extension(node): Extension<Arc<Node<S, P>>>,
    Query(params): Query<NonceReq>,
) -> impl IntoResponse
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
    info!("📣 >> next_account_nonce by: {:?}", params);
    let resp = json!({ "nonce": node.next_account_nonce(&params.account) });
    info!("📣 << next_account_nonce response: {:?}", resp);

    Json(resp)
}

#[derive(Debug, Deserialize)]
struct TxReq {
    from: String,
    to: String,
    value: u64,
    nonce: u64,
}

async fn transfer<S, P>(
    Extension(node): Extension<Arc<Node<S, P>>>,
    Json(tx): Json<TxReq>,
) -> Result<impl IntoResponse, HttpError>
where
    S: State + Send + Sync + 'static,
    P: Peer + Send + Sync + 'static,
{
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
    #[error("Forbidden: {0}")]
    Forbidden(Error),
    #[error("Not found: {0}")]
    NotFound(Error),
    #[error("Internal server error: {0}")]
    InternalServerError(Error),
}

impl From<Error> for HttpError {
    fn from(err: Error) -> Self {
        match err {
            Error::BadRequest(..) => HttpError::BadRequest(err),
            Error::InsufficientBalance(..) => HttpError::Forbidden(err),
            Error::BlockNotFound(..) => HttpError::NotFound(err),
            _ => HttpError::InternalServerError(err),
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        let status = match self {
            HttpError::BadRequest(_) => StatusCode::BAD_REQUEST,
            HttpError::Forbidden(_) => StatusCode::FORBIDDEN,
            HttpError::NotFound(_) => StatusCode::NOT_FOUND,
            HttpError::InternalServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({ "error": self.to_string() }));

        (status, body).into_response()
    }
}
