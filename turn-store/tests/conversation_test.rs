use anyhow::Context;
use jiff::Timestamp;
use turn_store::domain::model::{
    Conversation, ConversationPageParam, connect_database, create_conversation,
    get_conversation_by_id, page_conversations,
};
use uuid::Uuid;

#[tokio::test]
#[ignore = "需要设置 DATABASE_URL，并确保数据库已执行 migrations"]
async fn test_create_conversation() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").context("缺少 DATABASE_URL")?;
    let mut db = connect_database(&database_url).await?;
    let now = Timestamp::now();

    let conversation = create_conversation(
        &mut db,
        Conversation {
            id: Uuid::nil(),
            doc_id: "doc-1".to_string(),
            doc_type: "article".to_string(),
            user_id: 1,
            title: "标题".to_string(),
            r#type: "chat".to_string(),
            inline_type: None,
            created_at: now,
            updated_at: now,
            deleted_at: 0,
        },
    )
    .await?;

    println!("{:?}", conversation);
    let conversation = get_conversation_by_id(&mut db, conversation.id).await?;
    println!("{:?}", conversation);

    let page = page_conversations(
        &mut db,
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
