use std::{
    convert::Infallible,
    pin::Pin,
    time::{Duration, Instant},
};

use axum::response::sse::{Event, KeepAlive, KeepAliveStream, Sse};
use futures_util::Stream;
use serde_json::json;
use tokio::time::timeout;
use uuid::Uuid;

use crate::{
    app::AppState,
    domain::{
        event::AgentEvent,
        model::{Turn, UpstreamSpeed},
    },
    error::AppError,
    infra::redis_stream::StreamEntry,
};

#[derive(Debug, Clone, Copy)]
pub struct IdleDeadline {
    timeout: Duration,
    deadline: Instant,
}

impl IdleDeadline {
    pub fn new(now: Instant, timeout: Duration) -> Self {
        Self {
            timeout,
            deadline: now + timeout,
        }
    }

    pub fn reset(&mut self, now: Instant) {
        self.deadline = now + self.timeout;
    }

    pub fn remaining(&self, now: Instant) -> Option<Duration> {
        self.deadline.checked_duration_since(now)
    }
}

pub type TurnEventStream = Pin<Box<dyn Stream<Item = Result<Event, Infallible>> + Send + 'static>>;
pub type TurnSse = Sse<KeepAliveStream<TurnEventStream>>;

pub async fn start_turn_stream(
    state: AppState,
    conversation_id: Uuid,
    turn_id: Uuid,
    speed: UpstreamSpeed,
) -> Result<TurnSse, AppError> {
    let stream_key = Turn::stream_key(turn_id);
    let created = AgentEvent::turn_created(conversation_id, turn_id);
    state.redis_stream.append(&stream_key, &created).await?;
    let mut reader = state.redis_stream.reader().await?;
    state
        .relay_service
        .spawn(turn_id, stream_key.clone(), speed);
    let idle_timeout = state.relay_idle_timeout;

    let output = async_stream::stream! {
        let mut last_id = "0-0".to_string();
        let mut idle = IdleDeadline::new(Instant::now(), idle_timeout);

        'read: loop {
            let Some(remaining) = idle.remaining(Instant::now()) else {
                let data = AgentEvent::error("等待上游事件超时").data;
                yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                break;
            };

            match timeout(remaining, reader.read_after(&stream_key, &last_id)).await {
                Err(_) => {
                    let data = AgentEvent::error("等待上游事件超时").data;
                    yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                    break;
                }
                Ok(Err(error)) => {
                    eprintln!("读取 Redis Stream {stream_key} 失败: {error}");
                    let data = json!({
                        "type": "error",
                        "message": "读取 Redis Stream 失败"
                    })
                    .to_string();
                    yield Ok::<Event, Infallible>(Event::default().event("error").data(data));
                    break;
                }
                Ok(Ok(entries)) => {
                    if !entries.is_empty() {
                        idle.reset(Instant::now());
                    }
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
            }
        }
    };

    let output: TurnEventStream = Box::pin(output);
    Ok(Sse::new(output).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(10))
            .text("keep-alive"),
    ))
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::IdleDeadline;

    #[test]
    fn idle_deadline_only_moves_after_business_activity() {
        let start = Instant::now();
        let mut deadline = IdleDeadline::new(start, Duration::from_secs(60));

        assert_eq!(
            deadline.remaining(start + Duration::from_secs(15)),
            Some(Duration::from_secs(45))
        );
        deadline.reset(start + Duration::from_secs(20));
        assert_eq!(
            deadline.remaining(start + Duration::from_secs(50)),
            Some(Duration::from_secs(30))
        );
        assert_eq!(deadline.remaining(start + Duration::from_secs(81)), None);
    }
}
