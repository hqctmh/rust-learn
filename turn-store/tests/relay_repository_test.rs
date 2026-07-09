use sqlx::PgPool;
use uuid::Uuid;
use turn_store::{
    domain::{
        event::AgentEvent,
        model::{ConversationInput, TurnInput},
    },
    repositories::turn_store::{
        create_conversation_and_turn, create_turn_for_conversation, persist_event,
    },
};

#[sqlx::test]
async fn follow_up_turns_reuse_one_conversation(db: PgPool) -> sqlx::Result<()> {
    let (conversation, first_turn) = create_conversation_and_turn(
        &db,
        &ConversationInput {
            doc_id: "doc-multi".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "多轮对话".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        &TurnInput {
            input_context: "第一问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;

    let second_turn = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "第二问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;
    let third_turn = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "第三问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;

    let conversation_count =
        sqlx::query_scalar::<_, i64>("select count(*) from conversation where id = $1")
            .bind(conversation.id)
            .fetch_one(&db)
            .await?;
    let turn_count = sqlx::query_scalar::<_, i64>(
        "select count(*) from \"turn\" where conversation_id = $1",
    )
    .bind(conversation.id)
    .fetch_one(&db)
    .await?;

    assert_eq!(conversation_count, 1);
    assert_eq!(turn_count, 3);
    assert_eq!(first_turn.conversation_id, conversation.id);
    assert_eq!(second_turn.conversation_id, conversation.id);
    assert_eq!(third_turn.conversation_id, conversation.id);
    Ok(())
}

#[sqlx::test]
async fn follow_up_turn_rejects_missing_or_deleted_conversation(
    db: PgPool,
) -> sqlx::Result<()> {
    let missing = create_turn_for_conversation(
        &db,
        Uuid::nil(),
        &TurnInput {
            input_context: "不会写入".to_string(),
            document_content_version_id: 1,
        },
    )
    .await;
    assert!(matches!(missing, Err(sqlx::Error::RowNotFound)));

    let (conversation, _) = create_conversation_and_turn(
        &db,
        &ConversationInput {
            doc_id: "doc-deleted".to_string(),
            doc_type: "markdown".to_string(),
            user_id: 1,
            title: "已删除".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        &TurnInput {
            input_context: "第一问".to_string(),
            document_content_version_id: 1,
        },
    )
    .await?;
    sqlx::query("update conversation set deleted_at = 1 where id = $1")
        .bind(conversation.id)
        .execute(&db)
        .await?;

    let deleted = create_turn_for_conversation(
        &db,
        conversation.id,
        &TurnInput {
            input_context: "不会写入".to_string(),
            document_content_version_id: 1,
        },
    )
    .await;
    assert!(matches!(deleted, Err(sqlx::Error::RowNotFound)));
    Ok(())
}

#[sqlx::test]
async fn text_chunks_share_one_response_but_status_events_do_not(db: PgPool) -> sqlx::Result<()> {
    let (_, turn) = create_conversation_and_turn(
        &db,
        &ConversationInput {
            doc_id: "doc-1".to_string(),
            doc_type: "doc".to_string(),
            user_id: 1,
            title: "Redis Stream 测试".to_string(),
            r#type: "CHAT_EDIT".to_string(),
            inline_type: None,
        },
        &TurnInput {
            input_context: "测试 text 追加".to_string(),
            document_content_version_id: 1001,
        },
    )
    .await?;

    persist_event(
        &db,
        turn.id,
        &AgentEvent::from_sse("text", r#"{"type":"text","content":"你好"}"#),
    )
    .await?;
    persist_event(
        &db,
        turn.id,
        &AgentEvent::from_sse("text", r#"{"type":"text","content":"，Rust"}"#),
    )
    .await?;
    persist_event(
        &db,
        turn.id,
        &AgentEvent::from_sse("status", r#"{"type":"status","stage":"queued"}"#),
    )
    .await?;
    persist_event(
        &db,
        turn.id,
        &AgentEvent::from_sse("status", r#"{"type":"status","stage":"started"}"#),
    )
    .await?;

    let text_responses = sqlx::query_as::<_, (String, i64)>(
        r#"
        select response, count(*) over ()
        from turn_response
        where turn_id = $1 and type = 'text' and appendable
        "#,
    )
    .bind(turn.id)
    .fetch_one(&db)
    .await?;
    let status_count = sqlx::query_scalar::<_, i64>(
        r#"
        select count(*)
        from turn_response
        where turn_id = $1 and type = 'status' and not appendable
        "#,
    )
    .bind(turn.id)
    .fetch_one(&db)
    .await?;

    assert_eq!(text_responses, ("你好，Rust".to_string(), 1));
    assert_eq!(status_count, 2);
    Ok(())
}
