use axum::{
    Router,
    http::header::CONTENT_TYPE,
    response::{Html, IntoResponse, Response},
    routing::get,
};

use crate::app::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/styles.css", get(styles))
        .route("/sse.js", get(sse_script))
        .route("/run-events.js", get(run_events_script))
        .route("/app.js", get(app_script))
}

async fn index() -> Html<&'static str> {
    Html(include_str!("../../static/index.html"))
}

async fn styles() -> Response {
    asset(
        "text/css; charset=utf-8",
        include_str!("../../static/styles.css"),
    )
}

async fn sse_script() -> Response {
    asset(
        "text/javascript; charset=utf-8",
        include_str!("../../static/sse.js"),
    )
}

async fn run_events_script() -> Response {
    asset(
        "text/javascript; charset=utf-8",
        include_str!("../../static/run-events.js"),
    )
}

async fn app_script() -> Response {
    asset(
        "text/javascript; charset=utf-8",
        include_str!("../../static/app.js"),
    )
}

fn asset(content_type: &'static str, body: &'static str) -> Response {
    ([(CONTENT_TYPE, content_type)], body).into_response()
}
