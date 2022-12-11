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
    // 400
    #[error("Invalid peer address")]
    InvalidPeerAddress(#[from] AddrParseError),
    #[error("Invalid block number: expected '{0}', not '{0}'")]
    InvalidBlockNumber(u64, u64),
    #[error("Invalid block parent: expected '{0}', not '{0}'")]
    InvalidBlockParent(Hash, Hash),
    #[error("Block hash '{0}' donot meet the mining difficulty '{1}'")]
    InvalidBlockHash(Hash, usize),
    #[error("Invalid tx signature")]
    InvalidTxSignature(#[from] SignatureError),
    #[error("Invalid tx nonce: expected '{0}', not '{0}'")]
    InvalidTxNonce(u64, u64),

    // 403
    #[error("Insufficient balance: cost '{0}', balance is '{1}'")]
    InsufficientBalance(u64, u64),

    // 404
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    #[error("Block not found, number: {0}")]
    BlockNotFound(u64),

    // 500
    #[error("Failed to encode/decode message")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to access db")]
    DbError(#[from] std::io::Error),
}

// Tell axum how to convert `ChainError` into a response.
impl IntoResponse for ChainError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ChainError::InsufficientBalance(_, _) => (StatusCode::FORBIDDEN, self.to_string()),
            ChainError::AccountNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ChainError::BlockNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ChainError::JsonError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            ChainError::DbError(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
            _ => (StatusCode::BAD_REQUEST, self.to_string()),
        };

        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}
