use axum::{Json, Router, extract::State, routing::post};

use crate::{
    app::AppState,
    domain::model::CreateConversationStreamRequest,
    error::AppError,
    repositories::turn_store::create_conversation_and_turn,
    routes::stream::{TurnSse, start_turn_stream},
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/conversations/stream",
        post(create_conversation_stream),
    )
}

async fn create_conversation_stream(
    State(state): State<AppState>,
    Json(request): Json<CreateConversationStreamRequest>,
) -> Result<TurnSse, AppError> {
    request.validate().map_err(AppError::bad_request)?;
    let (conversation, turn) =
        create_conversation_and_turn(&state.db, &request.conversation, &request.turn).await?;
    start_turn_stream(state, conversation.id, turn.id, request.speed).await
}
