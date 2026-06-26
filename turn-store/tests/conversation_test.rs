use chrono::Utc;
use sqlx::PgPool;
use turn_store::domain::model::{
    Conversation, ConversationPageParam, create_conversation, get_conversation_by_id,
    page_conversations,
};
use uuid::Uuid;

#[sqlx::test(fixtures("conversation"))]
async fn test_create_conversation(db: PgPool) -> sqlx::Result<()> {
    let conversation = create_conversation(
        &db,
        Conversation {
            id: Uuid::nil(),
            doc_id: "doc-1".to_string(),
            doc_type: "article".to_string(),
            user_id: 1,
            title: "标题".to_string(),
            r#type: "chat".to_string(),
            inline_type: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: 0,
        },
    )
    .await?;

    println!("{:?}", conversation);
    let conversation = get_conversation_by_id(&db, conversation.id).await?;
    println!("{:?}", conversation);

    let page = page_conversations(
        &db,
        ConversationPageParam {
            user_id: Some(1),
            page: Some(1),
            page_size: Some(10),
            ..Default::default()
        },
    )
    .await?;

    println!("{:?}", page);
    Ok(())
}
