use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use turn_store::{
    app::{AppState, build_app},
    config::Config,
    infra::{redis_stream::RedisStream, upstream::UpstreamClient},
    services::relay::RelayService,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    let db = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;
    sqlx::migrate!().run(&db).await?;

    let redis_stream = RedisStream::connect(
        &config.redis_url,
        config.redis_stream_ttl_seconds,
        config.redis_xread_block_ms,
    )
    .await?;
    let http_client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .build()?;
    let upstream = UpstreamClient::new(http_client, config.upstream_agent_url);
    let relay_service = RelayService::new(db.clone(), redis_stream.clone(), upstream);

    let app = build_app(AppState {
        db,
        redis_stream,
        relay_service,
        relay_idle_timeout: Duration::from_secs(config.relay_idle_timeout_seconds),
    });

    let listener = tokio::net::TcpListener::bind(&config.bind_addr).await?;
    println!("turn-store listening on http://{}", config.bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
