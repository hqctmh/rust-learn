use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{
    event::{AgentEvent, PersistenceAction},
    model::{Conversation, ConversationInput, Turn, TurnInput},
};

pub async fn create_conversation_and_turn(
    db: &PgPool,
    conversation: &ConversationInput,
    turn: &TurnInput,
) -> sqlx::Result<(Conversation, Turn)> {
    let mut transaction = db.begin().await?;

    let conversation = sqlx::query_as::<_, Conversation>(
        r#"
        insert into conversation (doc_id, doc_type, user_id, title, type, inline_type)
        values ($1, $2, $3, $4, $5, $6)
        returning id, doc_id, doc_type, user_id, title, type, inline_type,
                  created_at, updated_at, deleted_at
        "#,
    )
    .bind(&conversation.doc_id)
    .bind(&conversation.doc_type)
    .bind(conversation.user_id)
    .bind(&conversation.title)
    .bind(&conversation.r#type)
    .bind(&conversation.inline_type)
    .fetch_one(&mut *transaction)
    .await?;

    let turn = sqlx::query_as::<_, Turn>(
        r#"
        insert into "turn" (conversation_id, input_context, document_content_version_id)
        values ($1, $2, $3)
        returning id, conversation_id, input_context, document_content_version_id,
                  created_at, updated_at, deleted_at
        "#,
    )
    .bind(conversation.id)
    .bind(&turn.input_context)
    .bind(turn.document_content_version_id)
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok((conversation, turn))
}

pub async fn create_turn_for_conversation(
    db: &PgPool,
    conversation_id: Uuid,
    turn: &TurnInput,
) -> sqlx::Result<Turn> {
    sqlx::query_as::<_, Turn>(
        r#"
        insert into "turn" (conversation_id, input_context, document_content_version_id)
        select id, $2, $3
        from conversation
        where id = $1 and deleted_at = 0
        returning id, conversation_id, input_context, document_content_version_id,
                  created_at, updated_at, deleted_at
        "#,
    )
    .bind(conversation_id)
    .bind(&turn.input_context)
    .bind(turn.document_content_version_id)
    .fetch_one(db)
    .await
}

pub async fn persist_event(db: &PgPool, turn_id: Uuid, event: &AgentEvent) -> sqlx::Result<()> {
    match event.persistence_action() {
        PersistenceAction::AppendText(content) => {
            sqlx::query(
                r#"
                insert into turn_response (turn_id, type, response, appendable)
                values ($1, 'text', $2, true)
                on conflict (turn_id, type) where appendable
                do update
                   set response = turn_response.response || excluded.response
                "#,
            )
            .bind(turn_id)
            .bind(content)
            .execute(db)
            .await?;
        }
        PersistenceAction::InsertRaw {
            event_type,
            response,
        } => {
            sqlx::query(
                r#"
                insert into turn_response (turn_id, type, response, appendable)
                values ($1, $2, $3, false)
                "#,
            )
            .bind(turn_id)
            .bind(event_type)
            .bind(response)
            .execute(db)
            .await?;
        }
    }

    Ok(())
}
