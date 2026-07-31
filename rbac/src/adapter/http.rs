use axum::{
    Json,
    extract::{Query, State},
};

use crate::{
    adapter::model::{ApiResponse, ApiResult, UserLoginParam, UserRegiestParam},
    di::AppState,
    domain::{
        dto::{LoginResponse, Page, UserPageQuery},
        model::User,
    },
};

pub async fn user_page_list(
    State(state): State<AppState>,
    Query(query): Query<UserPageQuery>,
) -> ApiResult<Page<User>> {
    let page = state.user_service.user_page_list(query).await?;
    Ok(Json(ApiResponse::success(page)))
}

#[axum::debug_handler]
pub async fn user_regist(
    State(state): State<AppState>,
    Json(body): Json<UserRegiestParam>,
) -> ApiResult<User> {
    let user = state
        .user_service
        .user_regist(&body.username, &body.password, body.display_name.as_deref())
        .await?;
    Ok(Json(ApiResponse::success(user)))
}

#[axum::debug_handler]
pub async fn user_login(
    State(state): State<AppState>,
    Json(body): Json<UserLoginParam>,
) -> ApiResult<LoginResponse> {
    let login_response = state
        .user_service
        .user_login(&body.username, &body.password, "")
        .await?;
    Ok(Json(ApiResponse::success(login_response)))
}
