use axum::{
    extract::{Extension, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use leptos::prelude::LeptosOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        auth::{LoginRequest, Session},
        comments::{CommentNode, CreateCommentRequest},
        posts::{CreatePostRequest, PostDetail, PostSummary},
        reactions::{FollowState, ToggleResult},
    },
    error::ForumError,
    state::ForumStore,
};

pub fn routes(store: ForumStore) -> Router<LeptosOptions> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/posts", get(list_posts).post(create_post))
        .route("/api/posts/{post_id}", get(get_post))
        .route(
            "/api/posts/{post_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route("/api/posts/{post_id}/like", post(toggle_like))
        .route("/api/posts/{post_id}/favorite", post(toggle_favorite))
        .route("/api/users/{user_id}/follow", post(follow_user))
        .layer(Extension(store))
}

async fn login(
    Extension(store): Extension<ForumStore>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(store.login(&request.username, &request.password)?))
}

async fn list_posts(Extension(store): Extension<ForumStore>) -> Json<Vec<PostSummary>> {
    Json(store.list_posts())
}

async fn create_post(
    Extension(store): Extension<ForumStore>,
    Json(request): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostDetail>), ApiError> {
    let user = store.demo_user();
    let detail = store.create_post(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn get_post(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<PostDetail>, ApiError> {
    Ok(Json(store.post_detail(post_id)?))
}

async fn list_comments(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<Vec<CommentNode>>, ApiError> {
    Ok(Json(store.comments_for_post(post_id)?))
}

async fn create_comment(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
    Json(mut request): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentNode>), ApiError> {
    request.post_id = post_id;
    let user = store.demo_user();
    let comment = store.add_comment(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn toggle_like(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user_id = request.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.toggle_post_like(user_id, post_id)?))
}

async fn toggle_favorite(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user_id = request.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.toggle_post_favorite(user_id, post_id)?))
}

async fn follow_user(
    Path(user_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<FollowState>, ApiError> {
    let follower_id = request.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.follow_user(follower_id, user_id)?))
}

#[derive(Clone, Debug, Deserialize)]
struct UserActionRequest {
    user_id: Option<Uuid>,
}

#[derive(Debug)]
struct ApiError(ForumError);

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    message: String,
}

impl From<ForumError> for ApiError {
    fn from(value: ForumError) -> Self {
        Self(value)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.0 {
            ForumError::Unauthorized => StatusCode::UNAUTHORIZED,
            ForumError::Forbidden => StatusCode::FORBIDDEN,
            ForumError::NotFound => StatusCode::NOT_FOUND,
            ForumError::Conflict(_) => StatusCode::CONFLICT,
            ForumError::Validation(_) => StatusCode::UNPROCESSABLE_ENTITY,
            ForumError::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = ApiErrorBody {
            message: self.0.to_string(),
        };
        (status, Json(body)).into_response()
    }
}
