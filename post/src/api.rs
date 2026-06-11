use axum::{
    Json, Router,
    extract::{
        Extension, Path, Query,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use leptos::prelude::LeptosOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        admin::AdminDashboard,
        announcements::{AnnouncementItem, AnnouncementReadState, CreateAnnouncementRequest},
        auth::{LoginRequest, RegisterRequest, Session},
        comments::{CommentNode, CreateCommentRequest},
        files::{FileAsset, FileUploadRequest},
        home::{HomePageData, HomeQuery, HomeSort, HomeTab, HomeTimeRange},
        moderation::{
            ModerationCommentAction, ModerationCommentRow, ModerationPostAction, ModerationPostRow,
        },
        notifications::{
            NotificationCenter, NotificationConnectionStats, NotificationPush,
            NotificationReadRequest,
        },
        posts::{
            AutosaveDraftRequest, CreatePostRequest, PostDetail, PostSummary, UpdatePostRequest,
        },
        rbac::{CreateRoleRequest, Permission, Role, UpdateRoleRequest},
        reactions::{FollowState, ToggleResult},
        reports::{CreateReportRequest, HandleReportRequest, ReportItem},
        search::{SearchQuery, SearchResultPage, SearchSort},
        taxonomy::{
            CategoryItem, CreateCategoryRequest, CreateTagRequest, MergeTagRequest, TagItem,
            UpdateCategoryRequest, UpdateTagRequest,
        },
        user_admin::{AdminUserRow, AuditContext, AuditLogEntry, UpdateUserRolesRequest},
        users::{
            ChangePasswordRequest, UpdateAvatarRequest, UpdateProfileRequest, UserProfile,
            UserSpace,
        },
    },
    error::ForumError,
    state::{AppState, ForumStore},
};

pub fn routes(state: AppState) -> Router<LeptosOptions> {
    let store = state.forum.clone();

    Router::new()
        .route("/api/home", get(home_page))
        .route("/api/categories", get(public_categories))
        .route("/api/tags", get(public_tags))
        .route("/api/announcements", get(public_announcements))
        .route(
            "/api/announcements/{announcement_id}/read",
            post(mark_announcement_read),
        )
        .route("/api/admin/dashboard", get(admin_dashboard))
        .route(
            "/api/admin/categories",
            get(list_admin_categories).post(create_admin_category),
        )
        .route(
            "/api/admin/categories/{category_id}/update",
            post(update_admin_category),
        )
        .route(
            "/api/admin/categories/{category_id}/disable",
            post(disable_admin_category),
        )
        .route(
            "/api/admin/tags",
            get(list_admin_tags).post(create_admin_tag),
        )
        .route("/api/admin/tags/{tag_id}/update", post(update_admin_tag))
        .route("/api/admin/tags/{tag_id}/merge", post(merge_admin_tag))
        .route("/api/admin/tags/{tag_id}/delete", post(delete_admin_tag))
        .route("/api/admin/posts", get(list_admin_posts))
        .route(
            "/api/admin/posts/{post_id}/offline",
            post(take_down_admin_post),
        )
        .route(
            "/api/admin/posts/{post_id}/restore",
            post(restore_admin_post),
        )
        .route("/api/admin/posts/{post_id}/delete", post(delete_admin_post))
        .route("/api/admin/posts/{post_id}/pin", post(pin_admin_post))
        .route("/api/admin/posts/{post_id}/unpin", post(unpin_admin_post))
        .route("/api/admin/comments", get(list_admin_comments))
        .route(
            "/api/admin/comments/{comment_id}/delete",
            post(delete_admin_comment),
        )
        .route(
            "/api/admin/comments/{comment_id}/recover",
            post(recover_admin_comment),
        )
        .route("/api/admin/users", get(list_admin_users))
        .route(
            "/api/admin/users/{user_id}/disable",
            post(disable_admin_user),
        )
        .route("/api/admin/users/{user_id}/enable", post(enable_admin_user))
        .route(
            "/api/admin/users/{user_id}/roles",
            post(update_admin_user_roles),
        )
        .route(
            "/api/admin/roles",
            get(list_admin_roles).post(create_admin_role),
        )
        .route(
            "/api/admin/roles/{role_code}/update",
            post(update_admin_role),
        )
        .route(
            "/api/admin/roles/{role_code}/delete",
            post(delete_admin_role),
        )
        .route("/api/admin/permissions", get(list_admin_permissions))
        .route("/api/admin/audit-logs", get(list_admin_audit_logs))
        .route(
            "/api/admin/announcements",
            get(list_admin_announcements).post(create_admin_announcement),
        )
        .route(
            "/api/admin/announcements/{announcement_id}/publish",
            post(publish_admin_announcement),
        )
        .route(
            "/api/admin/announcements/{announcement_id}/withdraw",
            post(withdraw_admin_announcement),
        )
        .route("/api/admin/reports", get(list_admin_reports))
        .route(
            "/api/admin/reports/{report_id}/handle",
            post(handle_admin_report),
        )
        .route("/api/files", post(upload_file))
        .route("/api/login", post(login))
        .route("/api/register", post(register))
        .route("/api/logout", post(logout))
        .route("/api/session/{session_id}", get(current_session))
        .route("/api/notifications", get(list_notifications))
        .route("/ws/notifications/{user_id}", get(notification_socket))
        .route("/api/notifications/online", get(notification_online_stats))
        .route(
            "/api/notifications/pending-pushes",
            get(list_pending_notification_pushes),
        )
        .route(
            "/api/notifications/pending-pushes/{push_id}/ack",
            post(ack_notification_push),
        )
        .route(
            "/api/notifications/read-all",
            post(mark_all_notifications_read),
        )
        .route(
            "/api/notifications/{notification_id}/read",
            post(mark_notification_read),
        )
        .route("/api/posts/drafts/autosave", post(autosave_draft))
        .route("/api/posts", get(list_posts).post(create_post))
        .route("/api/search", get(search))
        .route("/api/posts/{post_id}", get(get_post))
        .route("/api/posts/{post_id}/update", post(update_post))
        .route("/api/posts/{post_id}/delete", post(delete_own_post))
        .route(
            "/api/posts/{post_id}/comments",
            get(list_comments).post(create_comment),
        )
        .route(
            "/api/comments/{comment_id}/delete",
            post(delete_own_comment),
        )
        .route("/api/comments/{comment_id}/like", post(toggle_comment_like))
        .route("/api/comments/{comment_id}/report", post(report_comment))
        .route("/api/posts/{post_id}/like", post(toggle_like))
        .route("/api/posts/{post_id}/favorite", post(toggle_favorite))
        .route("/api/reports", post(create_report))
        .route("/api/users/{user_id}/profile", post(update_user_profile))
        .route("/api/users/{user_id}/avatar", post(update_user_avatar))
        .route("/api/users/{user_id}/password", post(change_user_password))
        .route("/api/users/{user_id}/follow", post(follow_user))
        .route("/api/users/{user_id}/space", get(user_space))
        .layer(Extension(store))
        .layer(Extension(state))
}

async fn login(
    Extension(state): Extension<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(
        state.login(&request.username, &request.password).await?,
    ))
}

async fn register(
    Extension(state): Extension<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<Session>), ApiError> {
    Ok((StatusCode::CREATED, Json(state.register(request).await?)))
}

async fn logout(
    Extension(state): Extension<AppState>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(state.logout(request.session_id).await?))
}

async fn current_session(
    Path(session_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<Json<Session>, ApiError> {
    Ok(Json(state.current_session(session_id).await?))
}

async fn list_posts(
    Extension(state): Extension<AppState>,
) -> Result<Json<Vec<PostSummary>>, ApiError> {
    Ok(Json(state.list_posts().await?))
}

async fn home_page(
    Extension(state): Extension<AppState>,
    Query(params): Query<HomeQueryParams>,
) -> Result<Json<HomePageData>, ApiError> {
    let user_id = params.user_id;
    Ok(Json(state.home_page(params.into(), user_id).await?))
}

async fn search(
    Extension(state): Extension<AppState>,
    Query(params): Query<SearchQueryParams>,
) -> Result<Json<SearchResultPage>, ApiError> {
    Ok(Json(state.search(params.into()).await?))
}

async fn admin_dashboard(
    Extension(store): Extension<ForumStore>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<AdminDashboard>, ApiError> {
    let user_id = params.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.admin_dashboard(user_id)?))
}

async fn public_categories(Extension(state): Extension<AppState>) -> Json<Vec<CategoryItem>> {
    Json(state.public_categories().await)
}

async fn public_tags(Extension(state): Extension<AppState>) -> Json<Vec<TagItem>> {
    Json(state.public_tags().await)
}

async fn list_admin_categories(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<CategoryItem>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_admin_categories(user_id).await?))
}

async fn create_admin_category(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<CreateCategoryRequest>,
) -> Result<(StatusCode, Json<CategoryItem>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let category = state.create_category(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(category)))
}

async fn update_admin_category(
    Path(category_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<UpdateCategoryRequest>,
) -> Result<Json<CategoryItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state.update_category(user_id, category_id, request).await?,
    ))
}

async fn disable_admin_category(
    Path(category_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<CategoryItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.disable_category(user_id, category_id).await?))
}

async fn list_admin_tags(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<TagItem>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_admin_tags(user_id).await?))
}

async fn create_admin_tag(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<TagItem>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let tag = state.create_tag(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(tag)))
}

async fn update_admin_tag(
    Path(tag_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<UpdateTagRequest>,
) -> Result<Json<TagItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.update_tag(user_id, tag_id, request).await?))
}

async fn merge_admin_tag(
    Path(tag_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<MergeTagRequest>,
) -> Result<Json<TagItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.merge_tag(user_id, tag_id, request).await?))
}

async fn delete_admin_tag(
    Path(tag_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<TagItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_tag(user_id, tag_id).await?))
}

async fn list_admin_posts(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<ModerationPostRow>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.admin_posts(user_id).await?))
}

async fn take_down_admin_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationPostAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.take_down_post(user_id, post_id).await?))
}

async fn restore_admin_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationPostAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.restore_post(user_id, post_id).await?))
}

async fn delete_admin_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationPostAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_post(user_id, post_id).await?))
}

async fn pin_admin_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationPostAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.pin_post(user_id, post_id).await?))
}

async fn unpin_admin_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationPostAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.unpin_post(user_id, post_id).await?))
}

async fn list_admin_comments(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<ModerationCommentRow>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.admin_comments(user_id).await?))
}

async fn delete_admin_comment(
    Path(comment_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationCommentAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_comment(user_id, comment_id).await?))
}

async fn recover_admin_comment(
    Path(comment_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<ModerationCommentAction>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.recover_comment(user_id, comment_id).await?))
}

async fn list_admin_users(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<AdminUserRow>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.admin_users(user_id).await?))
}

async fn disable_admin_user(
    Path(target_user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(context): Json<AuditContext>,
) -> Result<Json<AdminUserRow>, ApiError> {
    let admin_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state
            .disable_user(admin_id, target_user_id, context)
            .await?,
    ))
}

async fn enable_admin_user(
    Path(target_user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(context): Json<AuditContext>,
) -> Result<Json<AdminUserRow>, ApiError> {
    let admin_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state.enable_user(admin_id, target_user_id, context).await?,
    ))
}

async fn update_admin_user_roles(
    Path(target_user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<UpdateUserRolesRequest>,
) -> Result<Json<AdminUserRow>, ApiError> {
    let admin_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state
            .update_user_roles(admin_id, target_user_id, request)
            .await?,
    ))
}

async fn list_admin_roles(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<Role>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_roles(user_id).await?))
}

async fn create_admin_role(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<Role>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let role = state.create_role(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

async fn update_admin_role(
    Path(role_code): Path<String>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<UpdateRoleRequest>,
) -> Result<Json<Role>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.update_role(user_id, &role_code, request).await?))
}

async fn delete_admin_role(
    Path(role_code): Path<String>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(context): Json<AuditContext>,
) -> Result<Json<Role>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_role(user_id, &role_code, context).await?))
}

async fn list_admin_permissions(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<Permission>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_permissions(user_id).await?))
}

async fn list_admin_audit_logs(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<AuditLogEntry>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.audit_logs(user_id).await?))
}

async fn public_announcements(
    Extension(state): Extension<AppState>,
) -> Json<Vec<AnnouncementItem>> {
    Json(state.public_announcements().await)
}

async fn mark_announcement_read(
    Path(announcement_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<AnnouncementReadState>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state
            .mark_announcement_read(user_id, announcement_id)
            .await?,
    ))
}

async fn list_admin_announcements(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<AnnouncementItem>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_admin_announcements(user_id).await?))
}

async fn create_admin_announcement(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<CreateAnnouncementRequest>,
) -> Result<(StatusCode, Json<AnnouncementItem>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let announcement = state.create_announcement(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(announcement)))
}

async fn publish_admin_announcement(
    Path(announcement_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<AnnouncementItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state.publish_announcement(user_id, announcement_id).await?,
    ))
}

async fn withdraw_admin_announcement(
    Path(announcement_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<AnnouncementItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state
            .withdraw_announcement(user_id, announcement_id)
            .await?,
    ))
}

async fn list_admin_reports(
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
) -> Result<Json<Vec<ReportItem>>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.list_reports(user_id).await?))
}

async fn handle_admin_report(
    Path(report_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AdminDashboardQueryParams>,
    Json(request): Json<HandleReportRequest>,
) -> Result<Json<ReportItem>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state.handle_report(user_id, report_id, request).await?,
    ))
}

async fn upload_file(
    Extension(store): Extension<ForumStore>,
    Json(request): Json<FileUploadRequest>,
) -> Result<(StatusCode, Json<FileAsset>), ApiError> {
    let user = store.demo_user();
    let asset = store.upload_file(user.user_id, request)?;
    Ok((StatusCode::CREATED, Json(asset)))
}

async fn list_notifications(
    Extension(state): Extension<AppState>,
    Query(params): Query<NotificationQueryParams>,
) -> Result<Json<NotificationCenter>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.notification_center(user_id).await?))
}

async fn notification_socket(
    Path(user_id): Path<Uuid>,
    ws: WebSocketUpgrade,
    Extension(store): Extension<ForumStore>,
) -> Response {
    ws.on_upgrade(move |socket| notification_socket_session(socket, store, user_id))
}

async fn notification_socket_session(mut socket: WebSocket, store: ForumStore, user_id: Uuid) {
    if store.connect_notification_socket(user_id).is_err() {
        return;
    }

    if let Ok(pushes) = store.pending_notification_pushes(user_id) {
        for push in pushes {
            let text = format!(
                "notification:{}:{}",
                push.notification_id,
                push.title.replace('\n', " ")
            );
            if socket.send(Message::Text(text.into())).await.is_err() {
                let _ = store.disconnect_notification_socket(user_id);
                return;
            }
        }
    }

    while let Some(Ok(message)) = socket.recv().await {
        if let Message::Text(text) = message {
            if let Ok(push_id) = Uuid::parse_str(text.trim()) {
                let _ = store.ack_notification_push(user_id, push_id);
            }
        }
    }

    let _ = store.disconnect_notification_socket(user_id);
}

async fn notification_online_stats(
    Extension(store): Extension<ForumStore>,
    Query(params): Query<NotificationQueryParams>,
) -> Result<Json<NotificationConnectionStats>, ApiError> {
    let user_id = params.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.notification_connection_stats(user_id)?))
}

async fn list_pending_notification_pushes(
    Extension(store): Extension<ForumStore>,
    Query(params): Query<NotificationQueryParams>,
) -> Result<Json<Vec<NotificationPush>>, ApiError> {
    let user_id = params.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.pending_notification_pushes(user_id)?))
}

async fn ack_notification_push(
    Path(push_id): Path<Uuid>,
    Extension(store): Extension<ForumStore>,
    Json(request): Json<NotificationReadRequest>,
) -> Result<Json<NotificationConnectionStats>, ApiError> {
    let user_id = request.user_id.unwrap_or_else(|| store.demo_user().user_id);
    Ok(Json(store.ack_notification_push(user_id, push_id)?))
}

async fn mark_notification_read(
    Path(notification_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<NotificationReadRequest>,
) -> Result<Json<NotificationCenter>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(
        state
            .mark_notification_read(user_id, notification_id)
            .await?,
    ))
}

async fn mark_all_notifications_read(
    Extension(state): Extension<AppState>,
    Json(request): Json<NotificationReadRequest>,
) -> Result<Json<NotificationCenter>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.mark_all_notifications_read(user_id).await?))
}

#[derive(Clone, Debug, Default, Deserialize)]
struct HomeQueryParams {
    tab: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    time: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
    user_id: Option<Uuid>,
}

impl From<HomeQueryParams> for HomeQuery {
    fn from(value: HomeQueryParams) -> Self {
        Self {
            tab: parse_tab(value.tab.as_deref()),
            category: value.category,
            tag: value.tag,
            time: parse_time_range(value.time.as_deref()),
            sort: parse_sort(value.sort.as_deref()),
            page: value.page.unwrap_or(1),
            page_size: value.page_size.unwrap_or(12),
        }
    }
}

fn parse_tab(value: Option<&str>) -> HomeTab {
    match value {
        Some("hot") => HomeTab::Hot,
        Some("unanswered") => HomeTab::Unanswered,
        Some("following") => HomeTab::Following,
        _ => HomeTab::Latest,
    }
}

fn parse_time_range(value: Option<&str>) -> HomeTimeRange {
    match value {
        Some("today") => HomeTimeRange::Today,
        Some("week") => HomeTimeRange::Week,
        Some("month") => HomeTimeRange::Month,
        _ => HomeTimeRange::All,
    }
}

fn parse_sort(value: Option<&str>) -> HomeSort {
    match value {
        Some("created") => HomeSort::Created,
        Some("replies") => HomeSort::Replies,
        Some("views") => HomeSort::Views,
        Some("hot") => HomeSort::Hot,
        _ => HomeSort::LastReply,
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
struct SearchQueryParams {
    q: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct NotificationQueryParams {
    user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AdminDashboardQueryParams {
    user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct AuthorQueryParams {
    user_id: Option<Uuid>,
}

impl From<SearchQueryParams> for SearchQuery {
    fn from(value: SearchQueryParams) -> Self {
        Self {
            q: value.q.unwrap_or_default(),
            category: value.category,
            tag: value.tag,
            sort: parse_search_sort(value.sort.as_deref()),
            page: value.page.unwrap_or(1),
            page_size: value.page_size.unwrap_or(10),
        }
    }
}

fn parse_search_sort(value: Option<&str>) -> SearchSort {
    match value {
        Some("latest") => SearchSort::Latest,
        Some("hot") => SearchSort::Hot,
        _ => SearchSort::Relevance,
    }
}

async fn create_post(
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(request): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<PostDetail>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let detail = state.create_post(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn autosave_draft(
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(request): Json<AutosaveDraftRequest>,
) -> Result<(StatusCode, Json<PostDetail>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let detail = state.autosave_draft(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

async fn update_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(request): Json<UpdatePostRequest>,
) -> Result<Json<PostDetail>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.update_post(user_id, post_id, request).await?))
}

async fn delete_own_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
) -> Result<Json<PostDetail>, ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_own_post(user_id, post_id).await?))
}

async fn get_post(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<Json<PostDetail>, ApiError> {
    Ok(Json(state.post_detail(post_id).await?))
}

async fn list_comments(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
) -> Result<Json<Vec<CommentNode>>, ApiError> {
    Ok(Json(state.comments_for_post(post_id).await?))
}

async fn create_comment(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(mut request): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<CommentNode>), ApiError> {
    request.post_id = post_id;
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let comment = state.add_comment(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn delete_own_comment(
    Path(comment_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<CommentNode>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.delete_own_comment(user_id, comment_id).await?))
}

async fn toggle_comment_like(
    Path(comment_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.toggle_comment_like(user_id, comment_id).await?))
}

async fn report_comment(
    Path(comment_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(request): Json<CreateReportRequest>,
) -> Result<(StatusCode, Json<ReportItem>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let report = state.report_comment(user_id, comment_id, request).await?;
    Ok((StatusCode::CREATED, Json(report)))
}

async fn toggle_like(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.toggle_post_like(user_id, post_id).await?))
}

async fn toggle_favorite(
    Path(post_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<ToggleResult>, ApiError> {
    let user_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.toggle_post_favorite(user_id, post_id).await?))
}

async fn create_report(
    Extension(state): Extension<AppState>,
    Query(params): Query<AuthorQueryParams>,
    Json(request): Json<CreateReportRequest>,
) -> Result<(StatusCode, Json<ReportItem>), ApiError> {
    let user_id = params
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    let report = state.create_report(user_id, request).await?;
    Ok((StatusCode::CREATED, Json(report)))
}

async fn update_user_profile(
    Path(user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfile>, ApiError> {
    Ok(Json(state.update_profile(user_id, request).await?))
}

async fn update_user_avatar(
    Path(user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UpdateAvatarRequest>,
) -> Result<Json<UserProfile>, ApiError> {
    Ok(Json(state.update_avatar(user_id, request).await?))
}

async fn change_user_password(
    Path(user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<ChangePasswordRequest>,
) -> Result<StatusCode, ApiError> {
    state.change_password(user_id, request).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn follow_user(
    Path(user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Json(request): Json<UserActionRequest>,
) -> Result<Json<FollowState>, ApiError> {
    let follower_id = request
        .user_id
        .unwrap_or_else(|| state.forum.demo_user().user_id);
    Ok(Json(state.follow_user(follower_id, user_id).await?))
}

async fn user_space(
    Path(user_id): Path<Uuid>,
    Extension(state): Extension<AppState>,
    Query(params): Query<UserSpaceQueryParams>,
) -> Result<Json<UserSpace>, ApiError> {
    Ok(Json(state.user_space(user_id, params.viewer_id).await?))
}

#[derive(Clone, Debug, Deserialize)]
struct UserActionRequest {
    user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize)]
struct LogoutRequest {
    session_id: Uuid,
}

#[derive(Clone, Debug, Default, Deserialize)]
struct UserSpaceQueryParams {
    viewer_id: Option<Uuid>,
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
