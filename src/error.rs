use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChainError {
    #[error("Invalid peer address: {0}")]
    InvalidAddress(String),
    #[error("Block not found, number: {0}")]
    NotFound(u64),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

// Tell axum how to convert `ChainError` into a response.
impl IntoResponse for ChainError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            ChainError::InvalidAddress(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ChainError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ChainError::Unknown(_) => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({ "error": msg }));
        (status, body).into_response()
    }
}
