use std::{convert::Infallible, time::Duration};

use axum::{
    Json, Router,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::post,
};
use serde_json::json;

use crate::{
    app::AppState,
    domain::{
        event::AgentEvent,
        model::{CreateConversationStreamRequest, Turn},
    },
    error::AppError,
    infra::redis_stream::StreamEntry,
    repositories::turn_store::create_conversation_and_turn,
};

pub fn router() -> Router<AppState> {
    Router::new().route("/api/turns/stream", post(create_turn_stream))
}

async fn create_turn_stream(
    State(state): State<AppState>,
    Json(request): Json<CreateConversationStreamRequest>,
) -> Result<Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>>, AppError> {
    request.validate().map_err(AppError::bad_request)?;

    let (conversation, turn) = create_conversation_and_turn(
        &state.db,
        &request.conversation,
        &request.turn,
    )
    .await?;
    let stream_key = Turn::stream_key(turn.id);

    // Redis Stream 在第一次 XADD 时创建；首条事件把领域 ID 立即返回给前端。
    let created = AgentEvent::turn_created(conversation.id, turn.id);
    state.redis_stream.append(&stream_key, &created).await?;

    // 为该 SSE 响应建立独立的阻塞读连接，再启动上游生产者。
    let mut reader = state.redis_stream.reader().await?;
    state
        .relay_service
        .spawn(turn.id, stream_key.clone(), request.speed);

    let stream = async_stream::stream! {
        let mut last_id = "0-0".to_string();

        'read: loop {
            match reader.read_after(&stream_key, &last_id).await {
                Ok(entries) => {
                    for entry in entries {
                        let StreamEntry { id, event, data } = entry;
                        last_id = id.clone();
                        let terminal = AgentEvent::from_sse(&event, &data).is_terminal();

                        yield Ok::<Event, Infallible>(
                            Event::default().id(id).event(event).data(data)
                        );

                        if terminal {
                            break 'read;
                        }
                    }
                }
                Err(error) => {
                    eprintln!("读取 Redis Stream {stream_key} 失败: {error}");
                    let data = json!({
                        "type": "error",
                        "message": "读取 Redis Stream 失败",
                    })
                    .to_string();
                    yield Ok::<Event, Infallible>(
                        Event::default().event("error").data(data)
                    );
                    break;
                }
            }
        }
    };

    Ok(Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    ))
}
