use axum::{handler::Handler, http::StatusCode, response::IntoResponse, routing::get, Router};

use super::error::*;

pub fn new_router() -> Router {
    Router::new()
        .route("/", get(hello))
        .fallback(not_found.into_service())
}

async fn hello() -> Result<(), NodeError> {
    Ok(())
}

async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}
