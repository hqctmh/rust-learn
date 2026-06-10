use axum::{
    Json, Router,
    extract::{
        Extension, Path, Query,
        ws::{Message, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{delete, get, patch, post},
};
use leptos::prelude::LeptosOptions;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::{
    domain::{
        auth::{LoginRequest, Session, UpdateProfileRequest},
        auth::{RegisterRequest, SessionUser},
        comments::{CommentNode, CreateCommentRequest},
        files::{FileUploadRequest, StoredFile},
        moderation::{AdminStats, AuditLogEntry, CreateReportRequest, Report, ReportDecision},
        notifications::{Announcement, AnnouncementRequest, Notification},
        posts::{CreatePostRequest, PostDetail, PostStatus, PostSummary, UpdatePostRequest},
        posts::{SearchQuery, SearchResult},
        reactions::{FollowState, ToggleResult},
        taxonomy::{Category, Tag, TagInput, TaxonomyInput},
    },
    error::ForumError,
    state::ForumStore,
};

pub fn route_paths() -> Vec<&'static str> {
    vec![
        "/api/login",
        "/api/register",
        "/api/logout",
        "/api/me",
        "/api/users/me/profile",
        "/api/posts",
        "/api/posts/{post_id}",
        "/api/comments/{comment_id}",
        "/api/posts/{post_id}/comments",
        "/api/posts/{post_id}/like",
        "/api/posts/{post_id}/favorite",
        "/api/users/{user_id}/follow",
        "/api/categories",
        "/api/tags",
        "/api/search/posts",
        "/api/notifications",
        "/api/notifications/read-all",
        "/api/announcements",
        "/api/files",
        "/api/reports",
        "/api/reports/{report_id}/resolve",
        "/api/admin/users",
        "/api/admin/users/{user_id}/disabled",
        "/api/admin/posts/{post_id}/status",
        "/api/admin/comments/{comment_id}",
        "/api/admin/categories",
        "/api/admin/tags",
        "/api/admin/stats",
        "/api/admin/audit-logs",
        "/api/ws/notifications",
    ]
}

pub fn routes(store: ForumStore) -> Router<LeptosOptions> {
    Router::new()
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/logout", post(logout))
        .route("/api/me", get(current_user))
        .route("/api/users/me/profile", patch(update_profile))
        .route("/api/posts", get(list_posts).post(create_post))
        .route(
            "/api/posts/{post_id}",
            get(get_post).patch(update_post).delete(delete_post),
        )
        .route("/api/comments/{comment_id}", delete(delete_comment))
        .route(
            "/api/posts/{post_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route("/api/posts/{post_id}/like", post(toggle_like))
        .route("/api/posts/{post_id}/favorite", post(toggle_favorite))
        .route("/api/users/{user_id}/follow", post(follow_user))
        .route("/api/categories", get(list_categories))
        .route("/api/tags", get(list_tags))
        .route("/api/search/posts", get(search_posts))
        .route("/api/notifications", get(list_notifications))
        .route("/api/notifications/read-all", post(mark_notifications_read))
        .route("/api/announcements", post(publish_announcement))
        .route("/api/files", post(upload_file))
        .route("/api/reports", post(create_report))
        .route("/api/reports/{report_id}/resolve", post(resolve_report))
        .route("/api/admin/users", get(admin_users))
        .route(
            "/api/admin/users/{user_id}/disabled",
            patch(set_user_disabled),
        )
        .route("/api/admin/posts/{post_id}/status", patch(set_post_status))
        .route(
            "/api/admin/comments/{comment_id}",
            delete(admin_delete_comment),
        )
        .route("/api/admin/categories", post(create_category))
        .route("/api/admin/tags", post(create_tag))
        .route("/api/admin/stats", get(admin_stats))
        .route("/api/admin/audit-logs", get(audit_logs))
        .route("/api/ws/notifications", get(ws_notifications))
        .layer(Extension(store))
}

async fn login(
    Extension(store): Extension<ForumStore>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(store.login(&request.username, &request.password)?))
}

async fn register(
    Extension(store): Extension<ForumStore>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    Ok((StatusCode::CREATED, Json(store.register(request)?)))
}

async fn current_user(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<SessionUser>, ApiError> {
    Ok(Json(required_session_user(&store, &headers)?))
}

async fn logout(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<StatusCode, ApiError> {
    let session_id = session_id_from_headers(&headers)?;
    store.logout(session_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_profile(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<SessionUser>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.update_profile(user.user_id, request)?))
}

async fn list_posts(Extension(store): Extension<ForumStore>) -> Json<Vec<PostSummary>> {
    Json(store.list_posts())
}

async fn create_post(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostDetail>), ApiError> {
    let user = required_session_user(&store, &headers)?;
    let detail = store.create_post(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn get_post(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<PostDetail>, ApiError> {
    Ok(Json(store.post_detail(post_id)?))
}

async fn update_post(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UpdatePostRequest>,
) -> Result<Json<PostDetail>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.update_post(user.user_id, post_id, request)?))
}

async fn delete_post(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<PostDetail>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.delete_post(user.user_id, post_id)?))
}

async fn list_comments(
    Path(post_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<Vec<CommentNode>>, ApiError> {
    Ok(Json(store.comments_for_post(post_id)?))
}

async fn create_comment(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(mut request): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentNode>), ApiError> {
    request.post_id = post_id;
    let user = required_session_user(&store, &headers)?;
    let comment = store.add_comment(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn delete_comment(
    Path(comment_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<StatusCode, ApiError> {
    let user = required_session_user(&store, &headers)?;
    store.delete_comment(user.user_id, comment_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn toggle_like(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.toggle_post_like(user.user_id, post_id)?))
}

async fn toggle_favorite(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.toggle_post_favorite(user.user_id, post_id)?))
}

async fn follow_user(
    Path(user_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<FollowState>, ApiError> {
    let follower = required_session_user(&store, &headers)?;
    Ok(Json(store.follow_user(follower.user_id, user_id)?))
}

async fn list_categories(Extension(store): Extension<ForumStore>) -> Json<Vec<Category>> {
    Json(store.categories())
}

async fn list_tags(Extension(store): Extension<ForumStore>) -> Json<Vec<Tag>> {
    Json(store.tags())
}

async fn search_posts(
    Query(query): Query<SearchQuery>,
    Extension(store): Extension<ForumStore>,
) -> Json<SearchResult> {
    Json(store.search_posts(query))
}

async fn list_notifications(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<Vec<Notification>>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(store.list_notifications(user.user_id)))
}

async fn mark_notifications_read(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<MarkReadResponse>, ApiError> {
    let user = required_session_user(&store, &headers)?;
    Ok(Json(MarkReadResponse {
        changed: store.mark_all_notifications_read(user.user_id),
    }))
}

async fn publish_announcement(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<AnnouncementRequest>,
) -> Result<(StatusCode, Json<Announcement>), ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "announcement:publish")?;
    let announcement = store.publish_announcement(admin.user_id, request)?;
    Ok((StatusCode::CREATED, Json(announcement)))
}

async fn upload_file(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<FileUploadRequest>,
) -> Result<(StatusCode, Json<StoredFile>), ApiError> {
    let user = required_session_user(&store, &headers)?;
    let file = store.upload_file(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(file)))
}

async fn create_report(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(mut request): Json<CreateReportRequest>,
) -> Result<(StatusCode, Json<Report>), ApiError> {
    let user = required_session_user(&store, &headers)?;
    request.reporter_id = user.user_id;
    let report = store.create_report(request)?;
    Ok((StatusCode::CREATED, Json(report)))
}

async fn resolve_report(
    Path(report_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<ReportDecision>,
) -> Result<Json<Report>, ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "report:resolve")?;
    Ok(Json(store.resolve_report(
        admin.user_id,
        report_id,
        request,
    )?))
}

async fn admin_users(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<Vec<SessionUser>>, ApiError> {
    require_permission_from_headers(&store, &headers, "user:view")?;
    Ok(Json(store.admin_users()))
}

async fn set_user_disabled(
    Path(user_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<UserDisabledRequest>,
) -> Result<Json<SessionUser>, ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "user:disable")?;
    Ok(Json(store.set_user_disabled(
        admin.user_id,
        user_id,
        request.disabled,
    )?))
}

async fn set_post_status(
    Path(post_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<PostStatusRequest>,
) -> Result<Json<PostDetail>, ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "post:update")?;
    Ok(Json(store.set_post_status(
        admin.user_id,
        post_id,
        request.status,
    )?))
}

async fn admin_delete_comment(
    Path(comment_id): Path<Uuid>,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<StatusCode, ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "comment:delete")?;
    store.admin_delete_comment(admin.user_id, comment_id)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_category(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<TaxonomyInput>,
) -> Result<(StatusCode, Json<Category>), ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "category:create")?;
    Ok((
        StatusCode::CREATED,
        Json(store.create_category(admin.user_id, request)?),
    ))
}

async fn create_tag(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<TagInput>,
) -> Result<(StatusCode, Json<Tag>), ApiError> {
    let admin = require_permission_from_headers(&store, &headers, "tag:create")?;
    Ok((
        StatusCode::CREATED,
        Json(store.create_tag(admin.user_id, request)?),
    ))
}

async fn admin_stats(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<AdminStats>, ApiError> {
    require_permission_from_headers(&store, &headers, "stats:view")?;
    Ok(Json(store.admin_stats()))
}

async fn audit_logs(
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Result<Json<Vec<AuditLogEntry>>, ApiError> {
    require_permission_from_headers(&store, &headers, "audit:view")?;
    Ok(Json(store.audit_logs()))
}

pub fn notification_ws_initial_message(
    store: &ForumStore,
    user_id: Uuid,
) -> Result<String, serde_json::Error> {
    let notifications = store.list_notifications(user_id);
    let unread_count = notifications
        .iter()
        .filter(|notification| notification.read_at.is_none())
        .count();
    serde_json::to_string(&NotificationWsMessage {
        kind: "notification.snapshot",
        unread_count,
        notifications,
    })
}

async fn ws_notifications(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    Extension(store): Extension<ForumStore>,
) -> Response {
    let Ok(user) = required_session_user(&store, &headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };
    let user_id = user.user_id;
    ws.on_upgrade(move |mut socket| async move {
        let message = match notification_ws_initial_message(&store, user_id) {
            Ok(message) => message,
            Err(_) => return,
        };

        if socket.send(Message::Text(message.into())).await.is_err() {
            return;
        }

        let receiver = Arc::new(Mutex::new(store.subscribe_notifications(user_id)));
        loop {
            let receiver = receiver.clone();
            let notification = tokio::task::spawn_blocking(move || {
                receiver.lock().expect("notification receiver lock").recv()
            })
            .await;

            let Ok(Ok(notification)) = notification else {
                break;
            };
            let message = match serde_json::to_string(&NotificationWsMessage {
                kind: "notification.created",
                unread_count: 1,
                notifications: vec![notification],
            }) {
                Ok(message) => message,
                Err(_) => break,
            };
            if socket.send(Message::Text(message.into())).await.is_err() {
                break;
            }
        }
    })
}

fn required_session_user(
    store: &ForumStore,
    headers: &HeaderMap,
) -> Result<SessionUser, ForumError> {
    let session_id = session_id_from_headers(headers)?;
    store.current_user(session_id)
}

pub fn authorize_session_user(
    store: &ForumStore,
    session_id: Option<Uuid>,
) -> Result<SessionUser, ForumError> {
    let session_id = session_id.ok_or(ForumError::Unauthorized)?;
    store.current_user(session_id)
}

pub fn authorize_session_for_permission(
    store: &ForumStore,
    session_id: Option<Uuid>,
    permission_code: &str,
) -> Result<SessionUser, ForumError> {
    let user = authorize_session_user(store, session_id)?;
    store.require_permission(user.user_id, permission_code)?;
    Ok(user)
}

fn require_permission_from_headers(
    store: &ForumStore,
    headers: &HeaderMap,
    permission_code: &str,
) -> Result<SessionUser, ForumError> {
    let session_id = session_id_from_headers(headers)?;
    authorize_session_for_permission(store, Some(session_id), permission_code)
}

fn session_id_from_headers(headers: &HeaderMap) -> Result<Uuid, ForumError> {
    let value = headers
        .get("x-session-id")
        .and_then(|value| value.to_str().ok())
        .ok_or(ForumError::Unauthorized)?;
    Uuid::parse_str(value).map_err(|_| ForumError::Unauthorized)
}

#[derive(Debug, Serialize)]
struct MarkReadResponse {
    changed: usize,
}

#[derive(Debug, Deserialize)]
struct UserDisabledRequest {
    disabled: bool,
}

#[derive(Debug, Deserialize)]
struct PostStatusRequest {
    status: PostStatus,
}

#[derive(Debug, Serialize)]
struct NotificationWsMessage {
    kind: &'static str,
    unread_count: usize,
    notifications: Vec<Notification>,
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
