pub mod stream;
pub mod turn;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(turn::router())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
