use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::integrations::{IntegrationAction, SearchIndexMutation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IntegrationOutboxRow {
    pub outbox_id: Uuid,
    pub action_kind: String,
    pub subject: String,
    pub aggregate_id: Option<Uuid>,
    pub payload: String,
    pub status: String,
    pub attempts: i32,
    pub last_error: Option<String>,
    pub created_at: OffsetDateTime,
    pub processed_at: Option<OffsetDateTime>,
}

pub struct PostgresIntegrationRepository;

impl PostgresIntegrationRepository {
    pub async fn insert_actions(
        pool: &sqlx::PgPool,
        actions: &[IntegrationAction],
    ) -> sqlx::Result<Vec<IntegrationOutboxRow>> {
        let mut rows = Vec::with_capacity(actions.len());
        for action in actions {
            let action_kind = action_kind(action);
            let subject = action_subject(action);
            let aggregate_id = action_aggregate_id(action);
            let payload = action_payload(action);
            let row = sqlx::query_as!(
                IntegrationOutboxRow,
                r#"
insert into integration_outbox (
    action_kind,
    subject,
    aggregate_id,
    payload
)
values ($1, $2, $3, $4)
returning
    outbox_id,
    action_kind,
    subject,
    aggregate_id,
    payload,
    status,
    attempts,
    last_error,
    created_at,
    processed_at
"#,
                action_kind,
                subject,
                aggregate_id,
                payload
            )
            .fetch_one(pool)
            .await?;
            rows.push(row);
        }

        Ok(rows)
    }

    pub async fn list_pending(
        pool: &sqlx::PgPool,
        limit: i64,
    ) -> sqlx::Result<Vec<IntegrationOutboxRow>> {
        sqlx::query_as!(
            IntegrationOutboxRow,
            r#"
select
    outbox_id,
    action_kind,
    subject,
    aggregate_id,
    payload,
    status,
    attempts,
    last_error,
    created_at,
    processed_at
from integration_outbox
where status = 'pending'
order by created_at asc
limit $1
"#,
            limit
        )
        .fetch_all(pool)
        .await
    }

    pub async fn mark_processed(pool: &sqlx::PgPool, outbox_id: Uuid) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update integration_outbox
set status = 'processed',
    processed_at = coalesce(processed_at, now())
where outbox_id = $1
  and status = 'pending'
"#,
            outbox_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn mark_failed(
        pool: &sqlx::PgPool,
        outbox_id: Uuid,
        last_error: &str,
        max_attempts: i32,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update integration_outbox
set attempts = attempts + 1,
    last_error = $2,
    status = case when attempts + 1 >= $3 then 'failed' else 'pending' end
where outbox_id = $1
  and status = 'pending'
"#,
            outbox_id,
            last_error,
            max_attempts
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn action_kind(action: &IntegrationAction) -> &'static str {
    match action {
        IntegrationAction::CacheInvalidate(_) => "cache_invalidate",
        IntegrationAction::NatsPublish(_) => "nats_publish",
        IntegrationAction::SearchIndex(_) => "search_index",
    }
}

fn action_subject(action: &IntegrationAction) -> String {
    match action {
        IntegrationAction::CacheInvalidate(invalidation) => invalidation.reason.clone(),
        IntegrationAction::NatsPublish(event) => event.subject.clone(),
        IntegrationAction::SearchIndex(SearchIndexMutation::Upsert(document)) => {
            format!("search.{}.upsert", document.index)
        }
        IntegrationAction::SearchIndex(SearchIndexMutation::Delete { index, .. }) => {
            format!("search.{index}.delete")
        }
    }
}

fn action_aggregate_id(action: &IntegrationAction) -> Option<Uuid> {
    match action {
        IntegrationAction::CacheInvalidate(_) => None,
        IntegrationAction::NatsPublish(event) => Some(event.aggregate_id),
        IntegrationAction::SearchIndex(SearchIndexMutation::Upsert(document)) => {
            Some(document.document_id)
        }
        IntegrationAction::SearchIndex(SearchIndexMutation::Delete { document_id, .. }) => {
            Some(*document_id)
        }
    }
}

fn action_payload(action: &IntegrationAction) -> String {
    match action {
        IntegrationAction::CacheInvalidate(invalidation) => serde_json::json!({
            "kind": "cache_invalidate",
            "reason": invalidation.reason,
            "keys": invalidation.keys,
        })
        .to_string(),
        IntegrationAction::NatsPublish(event) => serde_json::json!({
            "kind": "nats_publish",
            "subject": event.subject,
            "aggregate_id": event.aggregate_id,
            "summary": event.payload_summary,
        })
        .to_string(),
        IntegrationAction::SearchIndex(SearchIndexMutation::Upsert(document)) => {
            serde_json::json!({
                "kind": "search_upsert",
                "index": document.index,
                "document_id": document.document_id,
                "title": document.title,
                "summary": document.summary,
                "body": document.body,
                "category_name": document.category_name,
                "tags": document.tags,
                "author_id": document.author_id,
            })
            .to_string()
        }
        IntegrationAction::SearchIndex(SearchIndexMutation::Delete { index, document_id }) => {
            serde_json::json!({
                "kind": "search_delete",
                "index": index,
                "document_id": document_id,
            })
            .to_string()
        }
    }
}
