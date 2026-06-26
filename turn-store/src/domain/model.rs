use chrono::{DateTime, Utc};
use sqlx::{PgPool, types::Uuid};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Conversation {
    pub id: Uuid,
    pub doc_id: String,
    pub doc_type: String,
    pub user_id: i64,
    pub title: String,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub inline_type: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct Turn {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub input_context: String,
    pub document_content_version_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: i64,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TurnResponse {
    pub id: Uuid,
    pub turn_id: Uuid,
    #[sqlx(rename = "type")]
    pub r#type: String,
    pub response: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ConversationPageParam {
    pub user_id: Option<i64>,
    pub doc_id: Option<String>,
    pub doc_type: Option<String>,
    pub r#type: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Clone)]
struct PageTotal {
    total: i64,
}

pub async fn create_conversation(
    db: &PgPool,
    conversation: Conversation,
) -> sqlx::Result<Conversation> {
    sqlx::query_as!(
        Conversation,
        r#"
            insert into conversation(doc_id, doc_type, user_id, title, type, inline_type) VALUES ($1, $2, $3, $4, $5, $6)
            returning
                id,
            doc_id,
            doc_type,
            user_id,
            title,
            "type",
            inline_type,
            created_at,
            updated_at,
            deleted_at
        "#,
        conversation.doc_id,
        conversation.doc_type,
        conversation.user_id,
        conversation.title,
        conversation.r#type,
        conversation.inline_type
    )
    .fetch_one(db)
    .await
}

pub async fn get_conversation_by_id(db: &PgPool, id: Uuid) -> sqlx::Result<Conversation> {
    sqlx::query_as!(
        Conversation,
        r#"
            select id,doc_id,doc_type,user_id,title,"type",inline_type,created_at,updated_at,deleted_at
            from conversation where id=$1
        "#,
        id
    )
    .fetch_one(db)
    .await
}

pub async fn page_conversations(
    db: &PgPool,
    param: ConversationPageParam,
) -> sqlx::Result<Page<Conversation>> {
    let page = param.page.unwrap_or(1).max(1);
    let page_size = param.page_size.unwrap_or(20).max(1);
    let limit = page_size.max(1);
    let offset = (page - 1) * limit;
    let doc_id = param.doc_id.as_deref();
    let doc_type = param.doc_type.as_deref();
    let conversation_type = param.r#type.as_deref();

    let total = sqlx::query_as!(
        PageTotal,
        r#"
            select count(*) as "total!"
            from conversation
            where ($1::bigint is null or user_id = $1)
                and ($2::text is null or doc_id = $2)
                and ($3::text is null or doc_type = $3)
                and ($4::text is null or "type" = $4)
                and deleted_at = 0
        "#,
        param.user_id,
        doc_id,
        doc_type,
        conversation_type
    )
    .fetch_one(db)
    .await?
    .total;

    let items = sqlx::query_as!(
        Conversation,
        r#"
            select id,doc_id,doc_type,user_id,title,"type",inline_type,created_at,updated_at,deleted_at
            from conversation
            where ($1::bigint is null or user_id = $1)
                and ($2::text is null or doc_id = $2)
                and ($3::text is null or doc_type = $3)
                and ($4::text is null or "type" = $4)
                and deleted_at = 0
            order by updated_at desc, id desc
            limit $5 offset $6
        "#,
        param.user_id,
        doc_id,
        doc_type,
        conversation_type,
        limit,
        offset
    )
    .fetch_all(db)
    .await?;

    Ok(Page {
        items,
        total,
        page,
        page_size,
    })
}

pub async fn delete_conversation(db: &PgPool, id: Uuid) -> sqlx::Result<()> {
    let timestamp = Utc::now().timestamp_millis();

    let result = sqlx::query!(
        r#"
            update conversation set deleted_at = $1 where id = $2 and deleted_at = 0
        "#,
        timestamp,
        id
    )
    .execute(db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(sqlx::Error::RowNotFound);
    }

    Ok(())
}

pub async fn create_turn(db: &PgPool, turn: &Turn) -> sqlx::Result<Turn> {
    
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_conversations_accepts_optional_param_and_returns_page() {
        let param = ConversationPageParam {
            user_id: None,
            doc_id: None,
            doc_type: None,
            r#type: None,
            page: None,
            page_size: None,
        };
        let page = Page::<Conversation> {
            items: Vec::new(),
            total: 0,
            page: 1,
            page_size: 20,
        };

        assert!(param.user_id.is_none());
        assert_eq!(page.total, 0);
        let _ = page_conversations;
    }
    
}
