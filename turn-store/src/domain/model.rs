use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub doc_id: String,
    pub doc_type: String,
    pub user_id: i64,
    pub title: String,
    pub r#type: String,
    pub inline_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConversationInput {
    pub doc_id: String,
    pub doc_type: String,
    pub user_id: i64,
    pub title: String,
    pub r#type: String,
    pub inline_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Turn {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub input_context: String,
    pub document_content_version_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: i64,
}

impl Turn {
    pub fn stream_key(turn_id: Uuid) -> String {
        format!("turn:{turn_id}:events")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TurnInput {
    pub input_context: String,
    pub document_content_version_id: i64,
}

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct TurnResponse {
    pub id: Uuid,
    pub turn_id: Uuid,
    pub r#type: String,
    pub response: String,
    pub appendable: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpstreamSpeed {
    #[default]
    Fast,
    Slow,
}

impl UpstreamSpeed {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Slow => "slow",
        }
    }
}

impl ConversationInput {
    pub fn validate(&self) -> Result<(), String> {
        required(&self.doc_id, "conversation.doc_id")?;
        max_chars(&self.doc_id, "conversation.doc_id", 64)?;
        required(&self.doc_type, "conversation.doc_type")?;
        max_chars(&self.doc_type, "conversation.doc_type", 50)?;
        required(&self.title, "conversation.title")?;
        max_chars(&self.title, "conversation.title", 255)?;
        required(&self.r#type, "conversation.type")?;
        max_chars(&self.r#type, "conversation.type", 50)?;
        if let Some(inline_type) = &self.inline_type {
            max_chars(inline_type, "conversation.inline_type", 50)?;
        }
        Ok(())
    }
}

impl TurnInput {
    pub fn validate(&self) -> Result<(), String> {
        required(&self.input_context, "turn.input_context")
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConversationStreamRequest {
    pub conversation: ConversationInput,
    pub turn: TurnInput,
    #[serde(default)]
    pub speed: UpstreamSpeed,
}

impl CreateConversationStreamRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.conversation.validate()?;
        self.turn.validate()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTurnStreamRequest {
    pub turn: TurnInput,
    #[serde(default)]
    pub speed: UpstreamSpeed,
}

impl CreateTurnStreamRequest {
    pub fn validate(&self) -> Result<(), String> {
        self.turn.validate()
    }
}

fn required(value: &str, field: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} 不能为空"));
    }
    Ok(())
}

fn max_chars(value: &str, field: &str, max: usize) -> Result<(), String> {
    if value.chars().count() > max {
        return Err(format!("{field} 不能超过 {max} 个字符"));
    }
    Ok(())
}

#[derive(Debug, Clone, Default)]
pub struct ConversationPageParam {
    pub doc_id: Option<String>,
    pub doc_type: Option<String>,
    pub user_id: Option<i64>,
    pub r#type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ConversationPage {
    pub records: Vec<Conversation>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn create_conversation(
    db: &PgPool,
    conversation: Conversation,
) -> sqlx::Result<Conversation> {
    sqlx::query_as::<_, Conversation>(
        r#"
        insert into conversation (doc_id, doc_type, user_id, title, type, inline_type)
        values ($1, $2, $3, $4, $5, $6)
        returning id, doc_id, doc_type, user_id, title, type, inline_type,
                  created_at, updated_at, deleted_at
        "#,
    )
    .bind(conversation.doc_id)
    .bind(conversation.doc_type)
    .bind(conversation.user_id)
    .bind(conversation.title)
    .bind(conversation.r#type)
    .bind(conversation.inline_type)
    .fetch_one(db)
    .await
}

pub async fn get_conversation_by_id(
    db: &PgPool,
    conversation_id: Uuid,
) -> sqlx::Result<Conversation> {
    sqlx::query_as::<_, Conversation>(
        r#"
        select id, doc_id, doc_type, user_id, title, type, inline_type,
               created_at, updated_at, deleted_at
        from conversation
        where id = $1 and deleted_at = 0
        "#,
    )
    .bind(conversation_id)
    .fetch_one(db)
    .await
}

pub async fn page_conversations(
    db: &PgPool,
    param: ConversationPageParam,
) -> sqlx::Result<ConversationPage> {
    let page = param.page.unwrap_or(1).max(1);
    let page_size = param.page_size.unwrap_or(20).clamp(1, 100);
    let offset = (page - 1) * page_size;

    let total = sqlx::query_scalar::<_, i64>(
        r#"
        select count(*)
        from conversation
        where deleted_at = 0
          and ($1::bigint is null or user_id = $1)
          and ($2::text is null or doc_id = $2)
          and ($3::text is null or doc_type = $3)
          and ($4::text is null or type = $4)
        "#,
    )
    .bind(param.user_id)
    .bind(param.doc_id.as_deref())
    .bind(param.doc_type.as_deref())
    .bind(param.r#type.as_deref())
    .fetch_one(db)
    .await?;

    let records = sqlx::query_as::<_, Conversation>(
        r#"
        select id, doc_id, doc_type, user_id, title, type, inline_type,
               created_at, updated_at, deleted_at
        from conversation
        where deleted_at = 0
          and ($1::bigint is null or user_id = $1)
          and ($2::text is null or doc_id = $2)
          and ($3::text is null or doc_type = $3)
          and ($4::text is null or type = $4)
        order by updated_at desc, id desc
        limit $5 offset $6
        "#,
    )
    .bind(param.user_id)
    .bind(param.doc_id.as_deref())
    .bind(param.doc_type.as_deref())
    .bind(param.r#type.as_deref())
    .bind(page_size)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok(ConversationPage {
        records,
        total,
        page,
        page_size,
    })
}
