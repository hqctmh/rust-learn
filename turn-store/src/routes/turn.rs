use axum::{
    Json, Router,
    extract::{Path, State},
    routing::post,
};
use uuid::Uuid;

use crate::{
    app::AppState,
    domain::model::CreateTurnStreamRequest,
    error::AppError,
    repositories::turn_store::create_turn_for_conversation,
    routes::stream::{TurnSse, start_turn_stream},
};

pub fn router() -> Router<AppState> {
    Router::new().route(
        "/api/conversations/{conversation_id}/turns/stream",
        post(create_turn_stream),
    )
}

async fn create_turn_stream(
    Path(conversation_id): Path<Uuid>,
    State(state): State<AppState>,
    Json(request): Json<CreateTurnStreamRequest>,
) -> Result<TurnSse, AppError> {
    request.validate().map_err(AppError::bad_request)?;
    let turn = create_turn_for_conversation(&state.db, conversation_id, &request.turn)
        .await
        .map_err(|error| match error {
            sqlx::Error::RowNotFound => AppError::not_found("Conversation 不存在或已删除"),
            error => error.into(),
        })?;
    start_turn_stream(state, conversation_id, turn.id, request.speed).await
}
