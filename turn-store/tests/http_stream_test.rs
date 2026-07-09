use std::{convert::Infallible, env, time::Duration};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header::CONTENT_TYPE},
    response::sse::{Event, Sse},
    routing::get,
};
use futures_util::stream;
use serde_json::{Value, json};
use sqlx::PgPool;
use tower::ServiceExt;
use turn_store::{
    app::{AppState, build_app},
    infra::{redis_stream::RedisStream, upstream::UpstreamClient},
    services::relay::RelayService,
};

async fn start_test_upstream() -> String {
    let app = Router::new().route(
        "/events",
        get(|| async {
            let events = stream::iter([
                Ok::<_, Infallible>(
                    Event::default()
                        .event("text")
                        .data(json!({"type":"text","content":"你好"}).to_string()),
                ),
                Ok::<_, Infallible>(
                    Event::default()
                        .event("text")
                        .data(json!({"type":"text","content":"，世界"}).to_string()),
                ),
                Ok::<_, Infallible>(
                    Event::default()
                        .event("run_completed")
                        .data(json!({"type":"run_completed","content":"完成"}).to_string()),
                ),
            ]);
            Sse::new(events)
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{address}/events")
}

async fn test_app(db: PgPool) -> Router {
    let redis_url =
        env::var("TEST_REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6380/15".to_string());
    let redis_stream = RedisStream::connect(&redis_url, 60, 50).await.unwrap();
    let upstream = UpstreamClient::new(reqwest::Client::new(), start_test_upstream().await);
    let relay_service = RelayService::new(db.clone(), redis_stream.clone(), upstream);
    build_app(AppState {
        db,
        redis_stream,
        relay_service,
        relay_idle_timeout: Duration::from_secs(5),
    })
}

async fn post_json(app: Router, uri: &str, body: Value) -> (StatusCode, String, String) {
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(uri)
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
    (
        status,
        content_type,
        String::from_utf8(body.to_vec()).unwrap(),
    )
}

fn event_names(body: &str) -> Vec<&str> {
    body.split("\n\n")
        .filter_map(|block| {
            block
                .lines()
                .find_map(|line| line.strip_prefix("event:").map(str::trim))
        })
        .collect()
}

fn event_data(body: &str, event_name: &str) -> Value {
    for block in body.split("\n\n") {
        let name = block
            .lines()
            .find_map(|line| line.strip_prefix("event:").map(str::trim));
        if name == Some(event_name) {
            let data = block
                .lines()
                .find_map(|line| line.strip_prefix("data:").map(str::trim))
                .unwrap();
            return serde_json::from_str(data).unwrap();
        }
    }
    panic!("没有找到 SSE 事件 {event_name}: {body}");
}

#[sqlx::test]
async fn first_and_follow_up_routes_reuse_conversation(db: PgPool) -> sqlx::Result<()> {
    let app = test_app(db.clone()).await;
    let first_request = json!({
        "conversation": {
            "doc_id": "web-doc",
            "doc_type": "markdown",
            "user_id": 1,
            "title": "网页对话",
            "type": "CHAT_EDIT",
            "inline_type": null
        },
        "turn": {
            "input_context": "第一问",
            "document_content_version_id": 1
        },
        "speed": "fast"
    });
    let (first_status, first_content_type, first_body) =
        post_json(app.clone(), "/api/conversations/stream", first_request).await;
    assert_eq!(first_status, StatusCode::OK);
    assert!(first_content_type.starts_with("text/event-stream"));
    assert_eq!(
        event_names(&first_body),
        ["turn_created", "text", "text", "run_completed"]
    );
    let created = event_data(&first_body, "turn_created");
    let conversation_id = created["conversation_id"].as_str().unwrap();

    let follow_up_request = json!({
        "turn": {
            "input_context": "第二问",
            "document_content_version_id": 1
        },
        "speed": "slow"
    });
    let uri = format!("/api/conversations/{conversation_id}/turns/stream");
    let (second_status, second_content_type, second_body) =
        post_json(app, &uri, follow_up_request).await;
    assert_eq!(second_status, StatusCode::OK);
    assert!(second_content_type.starts_with("text/event-stream"));
    assert_eq!(
        event_names(&second_body),
        ["turn_created", "text", "text", "run_completed"]
    );
    assert_eq!(
        event_data(&second_body, "turn_created")["conversation_id"].as_str(),
        Some(conversation_id)
    );

    let conversation_count = sqlx::query_scalar::<_, i64>("select count(*) from conversation")
        .fetch_one(&db)
        .await?;
    let turn_count = sqlx::query_scalar::<_, i64>("select count(*) from \"turn\"")
        .fetch_one(&db)
        .await?;
    let text_rows = sqlx::query_as::<_, (String, i64)>(
        "select response, count(*) over () from turn_response where type = 'text' and appendable order by created_at",
    )
    .fetch_all(&db)
    .await?;

    assert_eq!(conversation_count, 1);
    assert_eq!(turn_count, 2);
    assert_eq!(text_rows.len(), 2);
    assert!(
        text_rows
            .iter()
            .all(|(response, count)| response == "你好，世界" && *count == 2)
    );
    Ok(())
}

#[sqlx::test]
async fn follow_up_route_returns_404_for_missing_conversation(db: PgPool) {
    let app = test_app(db).await;
    let request = json!({
        "turn": {
            "input_context": "不会创建",
            "document_content_version_id": 1
        }
    });
    let (status, _, _) = post_json(
        app,
        "/api/conversations/00000000-0000-0000-0000-000000000000/turns/stream",
        request,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[sqlx::test]
async fn app_serves_chat_page_and_javascript(db: PgPool) {
    let app = test_app(db).await;
    let page = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(page.status(), StatusCode::OK);
    let page_body = to_bytes(page.into_body(), 1024 * 1024).await.unwrap();
    assert!(String::from_utf8_lossy(&page_body).contains("Turn Store Agent"));

    let script = app
        .oneshot(
            Request::builder()
                .uri("/app.js")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(script.status(), StatusCode::OK);
    assert_eq!(
        script.headers()[CONTENT_TYPE],
        "text/javascript; charset=utf-8"
    );
}
