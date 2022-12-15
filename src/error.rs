use std::net::AddrParseError;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use ethers_core::types::SignatureError;
use serde_json::json;
use thiserror::Error;

use crate::types::Hash;

#[derive(Error, Debug)]
pub enum ChainError {
    // 400: Bad Request
    #[error("Invalid peer address")]
    InvalidPeerAddress(#[from] AddrParseError),

    // 403: Forbidden
    #[error("Insufficient balance: cost '{0}', balance is '{1}'")]
    InsufficientBalance(u64, u64),

    // 404: Not Found
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Block not found, number: {0}")]
    BlockNotFound(u64),

    // 406: Not Acceptable (Sync Error)
    #[error("Invalid block number: expected '{0}', not '{1}'")]
    InvalidBlockNumber(u64, u64),
    #[error("Invalid block parent: expected '{0}', not '{1}'")]
    InvalidBlockParent(Hash, Hash),
    #[error("Block hash '{0}' donot meet the mining difficulty '{1}'")]
    InvalidBlockHash(Hash, usize),
    #[error("Invalid tx signature")]
    InvalidTxSignature(#[from] SignatureError),
    #[error("Invalid tx nonce: expected '{0}', not '{1}'")]
    InvalidTxNonce(u64, u64),

    // 500: Internal Server Error
    #[error("Failed to encode/decode message")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to access db")]
    DbError(#[from] std::io::Error),
    #[error("Failed to gossip with peer")]
    PeerError(#[from] reqwest::Error),
}

// Tell axum how to convert `ChainError` into a response.
impl IntoResponse for ChainError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ChainError::InvalidPeerAddress(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ChainError::InsufficientBalance(_, _) => (StatusCode::FORBIDDEN, self.to_string()),
            ChainError::AccountNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ChainError::BlockNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ChainError::InvalidBlockNumber(_, _) => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            ChainError::InvalidBlockParent(_, _) => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            ChainError::InvalidBlockHash(_, _) => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            ChainError::InvalidTxSignature(_) => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            ChainError::InvalidTxNonce(_, _) => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}
