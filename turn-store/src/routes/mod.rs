pub mod conversation;
pub mod stream;
pub mod turn;
pub mod web;

use axum::{Json, Router, routing::get};
use serde_json::{Value, json};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(conversation::router())
        .merge(turn::router())
        .merge(web::router())
}

async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}
