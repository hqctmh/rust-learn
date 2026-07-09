use anyhow::{Result, anyhow};
use eventsource_stream::Eventsource;
use futures_util::StreamExt;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{event::AgentEvent, model::UpstreamSpeed},
    infra::{redis_stream::RedisStream, upstream::UpstreamClient},
    repositories::turn_store::persist_event,
};

#[derive(Clone)]
pub struct RelayService {
    db: PgPool,
    redis_stream: RedisStream,
    upstream: UpstreamClient,
}

impl RelayService {
    pub fn new(db: PgPool, redis_stream: RedisStream, upstream: UpstreamClient) -> Self {
        Self {
            db,
            redis_stream,
            upstream,
        }
    }

    pub fn spawn(&self, turn_id: Uuid, stream_key: String, speed: UpstreamSpeed) {
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(error) = service.relay(turn_id, &stream_key, speed).await {
                service.publish_error(turn_id, &stream_key, error).await;
            }
        });
    }

    async fn relay(&self, turn_id: Uuid, stream_key: &str, speed: UpstreamSpeed) -> Result<()> {
        let response = self.upstream.connect(speed).await?;
        let mut upstream_events = response.bytes_stream().eventsource();
        let mut terminal_event_received = false;

        while let Some(result) = upstream_events.next().await {
            let upstream_event =
                result.map_err(|error| anyhow!("解析上游 SSE 事件失败: {error}"))?;
            let event = AgentEvent::from_sse(&upstream_event.event, &upstream_event.data);

            // 先持久化再投递，确保前端看到的事件已经落入 PostgreSQL。
            persist_event(&self.db, turn_id, &event).await?;
            self.redis_stream.append(stream_key, &event).await?;

            if event.is_terminal() {
                terminal_event_received = true;
                break;
            }
        }

        if !terminal_event_received {
            return Err(anyhow!("上游 SSE 在 run_completed 或 error 事件前结束"));
        }

        Ok(())
    }

    async fn publish_error(&self, turn_id: Uuid, stream_key: &str, error: anyhow::Error) {
        eprintln!("Turn {turn_id} 中转失败: {error:#}");
        let event = AgentEvent::error(error.to_string());

        if let Err(db_error) = persist_event(&self.db, turn_id, &event).await {
            eprintln!("保存 Turn {turn_id} 的 error 事件失败: {db_error}");
        }
        if let Err(redis_error) = self.redis_stream.append(stream_key, &event).await {
            eprintln!("投递 Turn {turn_id} 的 error 事件失败: {redis_error}");
        }
    }
}
