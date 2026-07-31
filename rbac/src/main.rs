use axum::{Router, routing::get};
use rbac::{adapter::http, di::AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let state = AppState::from_env().await?;
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    let app = create_router(state);
    axum::serve(listener, app).await?;

    Ok(())
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/user/page",get(http::user_page_list))
        .with_state(state)
}
