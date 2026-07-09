use std::time::Duration;

use axum::Router;
use sqlx::PgPool;

use crate::{infra::redis_stream::RedisStream, routes, services::relay::RelayService};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub redis_stream: RedisStream,
    pub relay_service: RelayService,
    pub relay_idle_timeout: Duration,
}

pub fn build_app(state: AppState) -> Router {
    routes::router().with_state(state)
}
