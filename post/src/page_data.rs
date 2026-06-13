use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    admin::{AdminDashboard, admin_dashboard_demo},
    auth::Session,
    comments::CommentNode,
    files::FileAsset,
    home::{HomePageData, HomeQuery, dense_workbench_home},
    notifications::{NotificationCenter, notification_demo_center},
    posts::{PostDetail, PostSummary},
    reactions::{FollowState, ToggleResult},
    reports::ReportItem,
    search::{SearchQuery, SearchResultPage, search_dense_workbench},
    users::{UserProfile, UserSpace},
};

#[cfg(feature = "ssr")]
fn server_error(message: impl ToString) -> ServerFnError {
    ServerFnError::ServerError(message.to_string())
}

#[cfg(feature = "ssr")]
fn parse_optional_announcement_time(
    value: &str,
) -> Result<Option<time::OffsetDateTime>, ServerFnError> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }

    let format = time::macros::format_description!("[year]-[month]-[day]T[hour]:[minute]");
    time::PrimitiveDateTime::parse(value, format)
        .map(|datetime| Some(datetime.assume_utc()))
        .map_err(server_error)
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostDetailPageData {
    pub post: PostDetail,
    pub comments: Vec<CommentNode>,
}

#[server]
pub async fn load_home_page(
    query: HomeQuery,
    session_id: String,
) -> Result<HomePageData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        let current_user_id = if session_id.trim().is_empty() {
            None
        } else {
            let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
            let session = state
                .current_session(session_id)
                .await
                .map_err(server_error)?;
            Some(session.user.user_id)
        };
        return state
            .home_page(query, current_user_id)
            .await
            .map_err(|error| ServerFnError::ServerError(error.to_string()));
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = query;
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_home_page() -> HomePageData {
    dense_workbench_home(HomeQuery::default(), false)
}

#[server]
pub async fn load_editor_post(
    session_id: String,
    post_id: String,
) -> Result<Option<PostDetail>, ServerFnError> {
    if post_id.trim().is_empty() {
        return Ok(None);
    }

    #[cfg(feature = "ssr")]
    {
        use crate::{error::ForumError, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        let post = state.post_detail(post_id).await.map_err(server_error)?;
        if post.summary.author_id != session.user.user_id {
            return Err(server_error(ForumError::Forbidden));
        }
        return Ok(Some(post));
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn login_user(username: String, password: String) -> Result<Session, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        return state
            .login(&username, &password)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (username, password);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn register_user(
    username: String,
    nickname: String,
    password: String,
    confirm_password: String,
) -> Result<Session, ServerFnError> {
    if password != confirm_password {
        return Err(ServerFnError::ServerError(
            "两次输入的密码不一致".to_string(),
        ));
    }

    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        return state
            .register(crate::domain::auth::RegisterRequest {
                username,
                password,
                nickname,
            })
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (username, nickname, password);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn submit_post(
    post_id: String,
    session_id: String,
    title: String,
    summary: String,
    category_name: String,
    tag_names: String,
    markdown: String,
    save_mode: String,
) -> Result<PostDetail, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = (!post_id.trim().is_empty())
            .then(|| Uuid::parse_str(post_id.trim()))
            .transpose()
            .map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        let tag_names = tag_names
            .split(',')
            .map(str::trim)
            .filter(|tag| !tag.is_empty())
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if save_mode == "draft" {
            return state
                .autosave_draft(
                    session.user.user_id,
                    crate::domain::posts::AutosaveDraftRequest {
                        post_id,
                        title,
                        markdown,
                        summary,
                        category_name: (!category_name.trim().is_empty())
                            .then(|| category_name.trim().to_string()),
                        tag_names,
                    },
                )
                .await
                .map_err(server_error);
        }

        if let Some(post_id) = post_id {
            return state
                .update_post(
                    session.user.user_id,
                    post_id,
                    crate::domain::posts::UpdatePostRequest {
                        title,
                        markdown,
                        summary,
                        category_name: (!category_name.trim().is_empty())
                            .then(|| category_name.trim().to_string()),
                        tag_names,
                        publish: true,
                    },
                )
                .await
                .map_err(server_error);
        }

        return state
            .create_post(
                session.user.user_id,
                crate::domain::posts::CreatePostRequest {
                    title,
                    markdown,
                    summary,
                    category_name: (!category_name.trim().is_empty())
                        .then(|| category_name.trim().to_string()),
                    tag_names,
                    publish: true,
                },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            post_id,
            session_id,
            title,
            summary,
            category_name,
            tag_names,
            markdown,
            save_mode,
        );
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn delete_editor_post(
    session_id: String,
    post_id: String,
) -> Result<PostDetail, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        let post = state
            .delete_own_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        if post.status != crate::domain::posts::PostStatus::Deleted {
            return Err(ServerFnError::ServerError("删除失败".to_string()));
        }
        return Ok(post);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn preview_markdown(markdown: String) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::services::posts::PostAuthoringService;

        return PostAuthoringService::preview_markdown(&markdown).map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = markdown;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn upload_editor_image(
    session_id: String,
    original_filename: String,
    mime_type: String,
    content_base64: String,
) -> Result<FileAsset, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::files::{FileBinaryUploadRequest, FileUsage},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;

        return state
            .upload_binary_file(
                session.user.user_id,
                FileBinaryUploadRequest {
                    original_filename,
                    mime_type,
                    content_base64,
                    usage: FileUsage::MarkdownImage,
                },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, original_filename, mime_type, content_base64);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn submit_comment(
    session_id: String,
    post_id: String,
    parent_comment_id: String,
    content: String,
) -> Result<CommentNode, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let parent_comment_id = (!parent_comment_id.trim().is_empty())
            .then(|| Uuid::parse_str(parent_comment_id.trim()))
            .transpose()
            .map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .add_comment(
                session.user.user_id,
                crate::domain::comments::CreateCommentRequest {
                    post_id,
                    parent_comment_id,
                    content,
                },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id, parent_comment_id, content);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn toggle_post_like(
    session_id: String,
    post_id: String,
) -> Result<ToggleResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .toggle_post_like(session.user.user_id, post_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn toggle_post_favorite(
    session_id: String,
    post_id: String,
) -> Result<ToggleResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .toggle_post_favorite(session.user.user_id, post_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn toggle_author_follow(
    session_id: String,
    author_id: String,
) -> Result<FollowState, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let author_id = Uuid::parse_str(&author_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .follow_user(session.user.user_id, author_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, author_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn toggle_comment_like(
    session_id: String,
    comment_id: String,
) -> Result<ToggleResult, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let comment_id = Uuid::parse_str(&comment_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .toggle_comment_like(session.user.user_id, comment_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, comment_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn delete_own_comment(
    session_id: String,
    comment_id: String,
) -> Result<CommentNode, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let comment_id = Uuid::parse_str(&comment_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .delete_own_comment(session.user.user_id, comment_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, comment_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn report_post(
    session_id: String,
    target_id: String,
    reason: String,
    description: String,
) -> Result<ReportItem, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let target_id = Uuid::parse_str(&target_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .create_report(
                session.user.user_id,
                crate::domain::reports::CreateReportRequest {
                    target_type: crate::domain::reports::ReportTargetType::Post,
                    target_id,
                    reason,
                    description: (!description.trim().is_empty())
                        .then(|| description.trim().to_string()),
                },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, target_id, reason, description);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn report_comment(
    session_id: String,
    comment_id: String,
    reason: String,
    description: String,
) -> Result<ReportItem, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let comment_id = Uuid::parse_str(&comment_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .report_comment(
                session.user.user_id,
                comment_id,
                crate::domain::reports::CreateReportRequest {
                    target_type: crate::domain::reports::ReportTargetType::Comment,
                    target_id: comment_id,
                    reason,
                    description: (!description.trim().is_empty())
                        .then(|| description.trim().to_string()),
                },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, comment_id, reason, description);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn load_search_page(query: SearchQuery) -> Result<SearchResultPage, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        return state
            .search(query)
            .await
            .map_err(|error| ServerFnError::ServerError(error.to_string()));
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = query;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_search_page(query: SearchQuery) -> SearchResultPage {
    search_dense_workbench(query)
}

#[server]
pub async fn load_admin_dashboard(session_id: String) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        if !session_id.trim().is_empty() {
            let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
            let session = state
                .current_session(session_id)
                .await
                .map_err(server_error)?;
            return state
                .admin_dashboard(session.user.user_id)
                .await
                .map_err(server_error);
        }

        let user_id = state.forum.demo_user().user_id;
        return state.admin_dashboard(user_id).await.map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_admin_dashboard() -> AdminDashboard {
    admin_dashboard_demo()
}

#[cfg(feature = "ssr")]
fn role_permission_codes(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(feature = "ssr")]
fn role_codes(value: &str) -> Vec<String> {
    value
        .split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[server]
pub async fn create_admin_role(
    session_id: String,
    code: String,
    name: String,
    permission_codes: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::{rbac::CreateRoleRequest, user_admin::AuditContext},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .create_role(
                session.user.user_id,
                CreateRoleRequest {
                    code,
                    name,
                    permission_codes: role_permission_codes(&permission_codes),
                    context: AuditContext::default(),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, code, name, permission_codes);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_admin_role(
    session_id: String,
    role_code: String,
    name: String,
    permission_codes: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::{rbac::UpdateRoleRequest, user_admin::AuditContext},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_role(
                session.user.user_id,
                &role_code,
                UpdateRoleRequest {
                    name: Some(name),
                    permission_codes: Some(role_permission_codes(&permission_codes)),
                    context: AuditContext::default(),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, role_code, name, permission_codes);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn delete_admin_role(
    session_id: String,
    role_code: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::user_admin::AuditContext, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .delete_role(session.user.user_id, &role_code, AuditContext::default())
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, role_code);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn create_admin_category(
    session_id: String,
    name: String,
    color: String,
    sort_order: i32,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::CreateCategoryRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .create_category(
                session.user.user_id,
                CreateCategoryRequest {
                    name,
                    color,
                    sort_order,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, name, color, sort_order);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn enable_admin_category(
    session_id: String,
    category_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::UpdateCategoryRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let category_id = Uuid::parse_str(&category_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_category(
                session.user.user_id,
                category_id,
                UpdateCategoryRequest {
                    name: None,
                    color: None,
                    sort_order: None,
                    enabled: Some(true),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, category_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_admin_category(
    session_id: String,
    category_id: String,
    name: String,
    color: String,
    sort_order: i32,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::UpdateCategoryRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let category_id = Uuid::parse_str(&category_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_category(
                session.user.user_id,
                category_id,
                UpdateCategoryRequest {
                    name: Some(name),
                    color: Some(color),
                    sort_order: Some(sort_order),
                    enabled: None,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, category_id, name, color, sort_order);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn disable_admin_category(
    session_id: String,
    category_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let category_id = Uuid::parse_str(&category_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .disable_category(session.user.user_id, category_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, category_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn create_admin_tag(
    session_id: String,
    name: String,
    sort_order: i32,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::CreateTagRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .create_tag(session.user.user_id, CreateTagRequest { name, sort_order })
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, name, sort_order);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn enable_admin_tag(
    session_id: String,
    tag_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::UpdateTagRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let tag_id = Uuid::parse_str(&tag_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_tag(
                session.user.user_id,
                tag_id,
                UpdateTagRequest {
                    name: None,
                    sort_order: None,
                    enabled: Some(true),
                    use_count: None,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, tag_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_admin_tag(
    session_id: String,
    tag_id: String,
    name: String,
    sort_order: i32,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::UpdateTagRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let tag_id = Uuid::parse_str(&tag_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_tag(
                session.user.user_id,
                tag_id,
                UpdateTagRequest {
                    name: Some(name),
                    sort_order: Some(sort_order),
                    enabled: None,
                    use_count: None,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, tag_id, name, sort_order);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn disable_admin_tag(
    session_id: String,
    tag_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::UpdateTagRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let tag_id = Uuid::parse_str(&tag_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_tag(
                session.user.user_id,
                tag_id,
                UpdateTagRequest {
                    name: None,
                    sort_order: None,
                    enabled: Some(false),
                    use_count: None,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, tag_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn merge_admin_tag(
    session_id: String,
    source_tag_id: String,
    target_tag_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::taxonomy::MergeTagRequest, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let source_tag_id = Uuid::parse_str(&source_tag_id).map_err(server_error)?;
        let target_tag_id = Uuid::parse_str(&target_tag_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .merge_tag(
                session.user.user_id,
                source_tag_id,
                MergeTagRequest { target_tag_id },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, source_tag_id, target_tag_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn create_admin_announcement(
    session_id: String,
    title: String,
    content: String,
    announcement_type: String,
    pinned: bool,
    effective_at: String,
    expires_at: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::announcements::{AnnouncementAudience, CreateAnnouncementRequest},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .create_announcement(
                session.user.user_id,
                CreateAnnouncementRequest {
                    title,
                    content,
                    announcement_type,
                    pinned,
                    effective_at: parse_optional_announcement_time(&effective_at)?,
                    expires_at: parse_optional_announcement_time(&expires_at)?,
                    audience: AnnouncementAudience::AllUsers,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            session_id,
            title,
            content,
            announcement_type,
            pinned,
            effective_at,
            expires_at,
        );
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_admin_announcement(
    session_id: String,
    announcement_id: String,
    title: String,
    content: String,
    announcement_type: String,
    pinned: bool,
    effective_at: String,
    expires_at: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::announcements::{AnnouncementAudience, UpdateAnnouncementRequest},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let announcement_id = Uuid::parse_str(&announcement_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_announcement(
                session.user.user_id,
                announcement_id,
                UpdateAnnouncementRequest {
                    title,
                    content,
                    announcement_type,
                    pinned,
                    effective_at: parse_optional_announcement_time(&effective_at)?,
                    expires_at: parse_optional_announcement_time(&expires_at)?,
                    audience: AnnouncementAudience::AllUsers,
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (
            session_id,
            announcement_id,
            title,
            content,
            announcement_type,
            pinned,
            effective_at,
            expires_at,
        );
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn push_admin_announcement(
    session_id: String,
    announcement_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let announcement_id = Uuid::parse_str(&announcement_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .push_announcement(session.user.user_id, announcement_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, announcement_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn publish_admin_announcement(
    session_id: String,
    announcement_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let announcement_id = Uuid::parse_str(&announcement_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .publish_announcement(session.user.user_id, announcement_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, announcement_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn withdraw_admin_announcement(
    session_id: String,
    announcement_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let announcement_id = Uuid::parse_str(&announcement_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .withdraw_announcement(session.user.user_id, announcement_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, announcement_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn disable_admin_user(
    session_id: String,
    target_user_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::user_admin::AuditContext, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let target_user_id = Uuid::parse_str(&target_user_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .disable_user(
                session.user.user_id,
                target_user_id,
                AuditContext::default(),
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, target_user_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn enable_admin_user(
    session_id: String,
    target_user_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{domain::user_admin::AuditContext, state::AppState};

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let target_user_id = Uuid::parse_str(&target_user_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .enable_user(
                session.user.user_id,
                target_user_id,
                AuditContext::default(),
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, target_user_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_admin_user_roles(
    session_id: String,
    target_user_id: String,
    roles: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::user_admin::{AuditContext, UpdateUserRolesRequest},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let target_user_id = Uuid::parse_str(&target_user_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .update_user_roles(
                session.user.user_id,
                target_user_id,
                UpdateUserRolesRequest {
                    roles: role_codes(&roles),
                    context: AuditContext::default(),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, target_user_id, roles);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn take_down_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .take_down_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn restore_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .restore_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn delete_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .delete_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn pin_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .pin_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn unpin_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .unpin_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn lock_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .lock_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn unlock_admin_post(
    session_id: String,
    post_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .unlock_post(session.user.user_id, post_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, post_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn delete_admin_comment(
    session_id: String,
    comment_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let comment_id = Uuid::parse_str(&comment_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .delete_comment(session.user.user_id, comment_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, comment_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn recover_admin_comment(
    session_id: String,
    comment_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let comment_id = Uuid::parse_str(&comment_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .recover_comment(session.user.user_id, comment_id)
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, comment_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn handle_admin_report(
    session_id: String,
    report_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::reports::{HandleReportRequest, ReportStatus},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let report_id = Uuid::parse_str(&report_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .handle_report(
                session.user.user_id,
                report_id,
                HandleReportRequest {
                    status: ReportStatus::Handled,
                    note: "已处理举报".to_string(),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, report_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn reject_admin_report(
    session_id: String,
    report_id: String,
) -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::{
            domain::reports::{HandleReportRequest, ReportStatus},
            state::AppState,
        };

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let report_id = Uuid::parse_str(&report_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .handle_report(
                session.user.user_id,
                report_id,
                HandleReportRequest {
                    status: ReportStatus::Rejected,
                    note: "已驳回举报".to_string(),
                },
            )
            .await
            .map_err(server_error)?;
        return state
            .admin_dashboard(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, report_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn load_notifications_page(
    session_id: String,
) -> Result<NotificationCenter, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        if !session_id.trim().is_empty() {
            let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
            let session = state
                .current_session(session_id)
                .await
                .map_err(server_error)?;
            return state
                .notification_center(session.user.user_id)
                .await
                .map_err(server_error);
        }

        let user_id = state.forum.demo_user().user_id;
        return state
            .notification_center(user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_notifications_page() -> NotificationCenter {
    notification_demo_center()
}

#[server]
pub async fn mark_page_notification_read(
    session_id: String,
    notification_id: String,
) -> Result<NotificationCenter, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let notification_id = Uuid::parse_str(&notification_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .mark_notification_read(session.user.user_id, notification_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, notification_id);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn mark_all_page_notifications_read(
    session_id: String,
) -> Result<NotificationCenter, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .mark_all_notifications_read(session.user.user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn load_post_detail_page(
    post_id: String,
    session_id: String,
) -> Result<PostDetailPageData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let current_user_id = if session_id.trim().is_empty() {
            None
        } else {
            let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
            let session = state
                .current_session(session_id)
                .await
                .map_err(server_error)?;
            Some(session.user.user_id)
        };
        let post = state
            .post_detail_for_user(post_id, current_user_id)
            .await
            .map_err(server_error)?;
        let comments = state
            .comments_for_post(post_id)
            .await
            .map_err(server_error)?;
        return Ok(PostDetailPageData { post, comments });
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = post_id;
        let _ = session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_post_detail_page(post_id: String) -> PostDetailPageData {
    let mut summary = PostSummary::sample();
    if let Ok(post_id) = Uuid::parse_str(&post_id) {
        summary.post_id = post_id;
    }
    PostDetailPageData {
        post: PostDetail {
            summary,
            markdown: "内容加载中。".to_string(),
            sanitized_html: "<p>内容加载中。</p>".to_string(),
            status: crate::domain::posts::PostStatus::Published,
            locked: false,
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        },
        comments: Vec::new(),
    }
}

#[server]
pub async fn load_user_space_page(
    profile_user_id: String,
    viewer_session_id: Option<String>,
) -> Result<UserSpace, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        let viewer_session_id = viewer_session_id.filter(|value| !value.trim().is_empty());
        let viewer_session = match viewer_session_id {
            Some(session_id) => {
                let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
                Some(
                    state
                        .current_session(session_id)
                        .await
                        .map_err(server_error)?,
                )
            }
            None => None,
        };

        if profile_user_id == "me" || profile_user_id == "sample" || profile_user_id.is_empty() {
            let user_id = viewer_session
                .as_ref()
                .map(|session| session.user.user_id)
                .unwrap_or_else(|| state.forum.demo_user().user_id);
            return state
                .user_space(user_id, Some(user_id))
                .await
                .map_err(server_error);
        }

        let profile_user_id = match Uuid::parse_str(&profile_user_id) {
            Ok(user_id) => user_id,
            Err(error) => return Err(server_error(error)),
        };
        let viewer_user_id = viewer_session.map(|session| session.user.user_id);
        return state
            .user_space(profile_user_id, viewer_user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = profile_user_id;
        let _ = viewer_session_id;
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_me_profile(
    session_id: String,
    nickname: String,
    bio: String,
) -> Result<UserProfile, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .update_profile(
                session.user.user_id,
                crate::domain::users::UpdateProfileRequest { nickname, bio },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, nickname, bio);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn update_me_avatar(
    session_id: String,
    avatar_url: String,
) -> Result<UserProfile, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        return state
            .update_avatar(
                session.user.user_id,
                crate::domain::users::UpdateAvatarRequest { avatar_url },
            )
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, avatar_url);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

#[server]
pub async fn change_me_password(
    session_id: String,
    old_password: String,
    new_password: String,
) -> Result<String, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let session_id = Uuid::parse_str(&session_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let session = state
            .current_session(session_id)
            .await
            .map_err(server_error)?;
        state
            .change_password(
                session.user.user_id,
                crate::domain::users::ChangePasswordRequest {
                    old_password,
                    new_password,
                },
            )
            .await
            .map_err(server_error)?;
        return Ok("密码更新成功".to_string());
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = (session_id, old_password, new_password);
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_user_space_page() -> UserSpace {
    let profile = crate::domain::users::UserProfile {
        user_id: Uuid::nil(),
        username: "loading".to_string(),
        nickname: "加载中".to_string(),
        avatar_url: None,
        bio: "用户空间加载中。".to_string(),
        registered_at: time::OffsetDateTime::now_utc(),
    };
    UserSpace {
        profile,
        stats: crate::domain::users::UserStats::default(),
        is_me: false,
        followed_by_viewer: false,
        published_posts: Vec::new(),
        draft_posts: Vec::new(),
        comments: Vec::new(),
        favorite_posts: Vec::new(),
        following: Vec::new(),
        followers: Vec::new(),
    }
}
