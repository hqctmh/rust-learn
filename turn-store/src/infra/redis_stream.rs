use redis::{Client, RedisResult, aio::ConnectionManager, streams::StreamReadReply};

use crate::domain::event::AgentEvent;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StreamEntry {
    pub id: String,
    pub event: String,
    pub data: String,
}

#[derive(Clone)]
pub struct RedisStream {
    client: Client,
    writer: ConnectionManager,
    ttl_seconds: u64,
    read_block_ms: usize,
}

impl RedisStream {
    pub async fn connect(
        redis_url: &str,
        ttl_seconds: u64,
        read_block_ms: usize,
    ) -> RedisResult<Self> {
        let client = Client::open(redis_url)?;
        let writer = client.get_connection_manager().await?;

        Ok(Self {
            client,
            writer,
            ttl_seconds,
            read_block_ms,
        })
    }

    pub async fn append(&self, stream_key: &str, event: &AgentEvent) -> RedisResult<String> {
        let mut writer = self.writer.clone();
        let entry_id: String = redis::cmd("XADD")
            .arg(stream_key)
            .arg("*")
            .arg("event")
            .arg(&event.event)
            .arg("data")
            .arg(&event.data)
            .query_async(&mut writer)
            .await?;

        let _: i64 = redis::cmd("EXPIRE")
            .arg(stream_key)
            .arg(self.ttl_seconds)
            .query_async(&mut writer)
            .await?;

        Ok(entry_id)
    }

    pub async fn reader(&self) -> RedisResult<RedisStreamReader> {
        // XREAD BLOCK 必须使用独立连接，避免阻塞生产者的 XADD。
        let connection = self.client.get_connection_manager().await?;
        Ok(RedisStreamReader {
            connection,
            read_block_ms: self.read_block_ms,
        })
    }
}

pub struct RedisStreamReader {
    connection: ConnectionManager,
    read_block_ms: usize,
}

impl RedisStreamReader {
    pub async fn read_after(
        &mut self,
        stream_key: &str,
        last_id: &str,
    ) -> RedisResult<Vec<StreamEntry>> {
        let reply: Option<StreamReadReply> = redis::cmd("XREAD")
            .arg("COUNT")
            .arg(100)
            .arg("BLOCK")
            .arg(self.read_block_ms)
            .arg("STREAMS")
            .arg(stream_key)
            .arg(last_id)
            .query_async(&mut self.connection)
            .await?;

        let mut entries = Vec::new();
        for key in reply.into_iter().flat_map(|reply| reply.keys) {
            for id in key.ids {
                let event = id.get("event").unwrap_or_else(|| "message".to_string());
                let data = id.get("data").unwrap_or_default();
                entries.push(StreamEntry {
                    id: id.id,
                    event,
                    data,
                });
            }
        }

        Ok(entries)
    }
}
