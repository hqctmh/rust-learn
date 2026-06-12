use std::{future::Future, pin::Pin};

use elasticsearch::{DeleteParts, Elasticsearch, IndexParts, http::transport::Transport};
use serde_json::{Value, json};

use crate::{
    integration_worker::IntegrationActionHandler, repositories::integrations::IntegrationOutboxRow,
    state::RuntimeConfig,
};

pub struct RuntimeIntegrationHandler {
    redis: redis::Client,
    nats: async_nats::Client,
    elasticsearch: Elasticsearch,
}

impl RuntimeIntegrationHandler {
    pub async fn from_config(config: &RuntimeConfig) -> Result<Self, String> {
        let redis = redis::Client::open(config.redis_url.as_str())
            .map_err(|error| format!("Redis 客户端初始化失败: {error}"))?;
        let _connection = redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| format!("Redis 连接失败: {error}"))?;

        let nats = async_nats::connect(config.nats_url.as_str())
            .await
            .map_err(|error| format!("NATS 连接失败: {error}"))?;

        let transport = Transport::single_node(config.elasticsearch_url.as_str())
            .map_err(|error| format!("Elasticsearch transport 初始化失败: {error}"))?;
        let elasticsearch = Elasticsearch::new(transport);

        Ok(Self {
            redis,
            nats,
            elasticsearch,
        })
    }

    async fn handle_cache_invalidation(&self, row: &IntegrationOutboxRow) -> Result<(), String> {
        let payload = parse_payload(row)?;
        let keys = payload
            .get("keys")
            .and_then(Value::as_array)
            .ok_or_else(|| "cache invalidation payload missing keys".to_string())?;
        let mut connection = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| format!("Redis 连接失败: {error}"))?;

        for key in keys.iter().filter_map(Value::as_str) {
            if key.contains('*') {
                let matching_keys: Vec<String> = redis::cmd("KEYS")
                    .arg(key)
                    .query_async(&mut connection)
                    .await
                    .map_err(|error| format!("Redis KEYS 失败: {error}"))?;
                if !matching_keys.is_empty() {
                    let _: usize = redis::cmd("DEL")
                        .arg(matching_keys)
                        .query_async(&mut connection)
                        .await
                        .map_err(|error| format!("Redis DEL 失败: {error}"))?;
                }
                continue;
            }

            let _: usize = redis::cmd("DEL")
                .arg(key)
                .query_async(&mut connection)
                .await
                .map_err(|error| format!("Redis DEL 失败: {error}"))?;
        }

        Ok(())
    }

    async fn handle_nats_publish(&self, row: &IntegrationOutboxRow) -> Result<(), String> {
        self.nats
            .publish(row.subject.clone(), row.payload.clone().into())
            .await
            .map_err(|error| format!("NATS 发布失败: {error}"))?;
        self.nats
            .flush()
            .await
            .map_err(|error| format!("NATS flush 失败: {error}"))?;
        Ok(())
    }

    async fn handle_search_index(&self, row: &IntegrationOutboxRow) -> Result<(), String> {
        let payload = parse_payload(row)?;
        let index = required_str(&payload, "index")?;
        let document_id = required_str(&payload, "document_id")?;

        match payload.get("kind").and_then(Value::as_str) {
            Some("search_delete") => {
                self.elasticsearch
                    .delete(DeleteParts::IndexId(index, document_id))
                    .send()
                    .await
                    .map_err(|error| format!("Elasticsearch 删除失败: {error}"))?;
            }
            _ => {
                let body = json!({
                    "title": payload.get("title").cloned().unwrap_or(Value::Null),
                    "summary": payload.get("summary").cloned().unwrap_or(Value::Null),
                    "body": payload.get("body").cloned().unwrap_or(Value::Null),
                    "category_name": payload.get("category_name").cloned().unwrap_or(Value::Null),
                    "tags": payload.get("tags").cloned().unwrap_or(Value::Null),
                    "author_id": payload.get("author_id").cloned().unwrap_or(Value::Null),
                });
                self.elasticsearch
                    .index(IndexParts::IndexId(index, document_id))
                    .body(body)
                    .send()
                    .await
                    .map_err(|error| format!("Elasticsearch 写入失败: {error}"))?;
            }
        }

        Ok(())
    }
}

impl IntegrationActionHandler for RuntimeIntegrationHandler {
    fn handle<'a>(
        &'a self,
        row: &'a IntegrationOutboxRow,
    ) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
        Box::pin(async move {
            match row.action_kind.as_str() {
                "cache_invalidate" => self.handle_cache_invalidation(row).await,
                "nats_publish" => self.handle_nats_publish(row).await,
                "search_index" => self.handle_search_index(row).await,
                other => Err(format!("未知 integration action 类型: {other}")),
            }
        })
    }
}

fn parse_payload(row: &IntegrationOutboxRow) -> Result<Value, String> {
    serde_json::from_str(&row.payload).map_err(|error| {
        format!(
            "integration outbox payload JSON 解析失败 outbox_id={}: {error}",
            row.outbox_id
        )
    })
}

fn required_str<'a>(payload: &'a Value, field: &str) -> Result<&'a str, String> {
    payload
        .get(field)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("integration payload missing {field}"))
}
