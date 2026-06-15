#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use axum::Router;
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use post::api;
    use post::app::{App, shell};
    use post::integration_runtime::spawn_integration_outbox_worker;
    use post::repositories::demo_seed::PostgresDemoSeedRepository;
    use post::state::{AppState, ForumStore, RuntimeConfig};

    dotenvy::dotenv().ok();
    init_tracing();

    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let forum_store = ForumStore::seeded();
    let runtime_config = RuntimeConfig::from_env();
    let db = sqlx::PgPool::connect_lazy(&runtime_config.database_url).ok();
    if demo_seed_enabled() {
        if let Some(pool) = &db {
            if let Err(error) = PostgresDemoSeedRepository::ensure_homepage_seed(pool).await {
                log!("homepage demo seed skipped: {error}");
            }
        }
    }
    let _integration_worker = spawn_integration_outbox_worker(db.clone(), runtime_config);
    let app_state = AppState {
        db,
        forum: forum_store,
    };
    let page_state = app_state.clone();
    let shell_options = leptos_options.clone();

    let app = Router::new()
        .merge(api::routes(app_state))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(page_state.clone()),
            move || shell(shell_options.clone()),
        )
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(http_trace_layer())
        .with_state(leptos_options);

    log!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(feature = "ssr")]
fn demo_seed_enabled() -> bool {
    std::env::var("POST_DEMO_SEED_HOME")
        .map(|value| value == "true" || value == "1")
        .unwrap_or(false)
}

#[cfg(feature = "ssr")]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("post=info,tower_http=info,axum=info,sqlx=warn"));

    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_env_filter(env_filter)
        .try_init();
}

#[cfg(feature = "ssr")]
fn http_trace_layer() -> tower_http::trace::TraceLayer<tower_http::trace::HttpMakeClassifier> {
    use tower_http::{
        LatencyUnit,
        trace::{
            DefaultMakeSpan, DefaultOnFailure, DefaultOnRequest, DefaultOnResponse, TraceLayer,
        },
    };
    use tracing::Level;

    TraceLayer::new_for_http()
        .make_span_with(
            DefaultMakeSpan::new()
                .include_headers(true)
                .level(Level::INFO),
        )
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        )
        .on_failure(
            DefaultOnFailure::new()
                .level(Level::ERROR)
                .latency_unit(LatencyUnit::Micros),
        )
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
