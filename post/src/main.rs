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
    use post::state::{AppState, ForumStore, RuntimeConfig};

    dotenvy::dotenv().ok();

    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);
    let forum_store = ForumStore::seeded();
    let runtime_config = RuntimeConfig::from_env();
    let db = sqlx::PgPool::connect_lazy(&runtime_config.database_url).ok();
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
        .with_state(leptos_options);

    log!("listening on http://{addr}");
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
