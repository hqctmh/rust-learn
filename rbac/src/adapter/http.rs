use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    adapter::model::{ApiResponse, ApiResult},
    di::AppState,
    domain::{
        model::User,
        param::{Page, UserPageQuery},
    },
};

pub async fn user_page_list(
    State(state): State<AppState>,
    Query(query): Query<UserPageQuery>,
) -> ApiResult<Page<User>> {
    let page = state.user_service.user_page_list(query).await?;
    Ok(Json(ApiResponse::success(Some(page))))
}
