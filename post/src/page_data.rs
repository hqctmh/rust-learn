use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    admin::{AdminDashboard, admin_dashboard_demo},
    comments::CommentNode,
    home::{HomePageData, HomeQuery, dense_workbench_home},
    notifications::{NotificationCenter, notification_demo_center},
    posts::{PostDetail, PostSummary},
    search::{SearchQuery, SearchResultPage, search_dense_workbench},
    users::UserSpace,
};

#[cfg(feature = "ssr")]
fn server_error(message: impl ToString) -> ServerFnError {
    ServerFnError::ServerError(message.to_string())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostDetailPageData {
    pub post: PostDetail,
    pub comments: Vec<CommentNode>,
}

#[server]
pub async fn load_home_page(query: HomeQuery) -> Result<HomePageData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        return state
            .home_page(query, None)
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

pub fn fallback_home_page() -> HomePageData {
    dense_workbench_home(HomeQuery::default(), false)
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
pub async fn load_admin_dashboard() -> Result<AdminDashboard, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        let user_id = state.forum.demo_user().user_id;
        return state
            .admin_dashboard(user_id)
            .await
            .map_err(|error| ServerFnError::ServerError(error.to_string()));
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_admin_dashboard() -> AdminDashboard {
    admin_dashboard_demo()
}

#[server]
pub async fn load_notifications_page() -> Result<NotificationCenter, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        let user_id = state.forum.demo_user().user_id;
        return state
            .notification_center(user_id)
            .await
            .map_err(|error| ServerFnError::ServerError(error.to_string()));
    }

    #[cfg(not(feature = "ssr"))]
    {
        Err(ServerFnError::ServerError(
            "server function called outside SSR".to_string(),
        ))
    }
}

pub fn fallback_notifications_page() -> NotificationCenter {
    notification_demo_center()
}

#[server]
pub async fn load_post_detail_page(post_id: String) -> Result<PostDetailPageData, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let post_id = Uuid::parse_str(&post_id).map_err(server_error)?;
        let state = expect_context::<AppState>();
        let post = state.post_detail(post_id).await.map_err(server_error)?;
        let comments = state
            .comments_for_post(post_id)
            .await
            .map_err(server_error)?;
        return Ok(PostDetailPageData { post, comments });
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = post_id;
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
    viewer_user_id: Option<String>,
) -> Result<UserSpace, ServerFnError> {
    #[cfg(feature = "ssr")]
    {
        use crate::state::AppState;

        let state = expect_context::<AppState>();
        if profile_user_id == "me" || profile_user_id == "sample" || profile_user_id.is_empty() {
            let user_id = state.forum.demo_user().user_id;
            return state
                .forum
                .user_space(user_id, Some(user_id))
                .map_err(server_error);
        }

        let profile_user_id = match Uuid::parse_str(&profile_user_id) {
            Ok(user_id) => user_id,
            Err(error) => return Err(server_error(error)),
        };
        let viewer_user_id = viewer_user_id
            .filter(|value| !value.is_empty())
            .map(|value| Uuid::parse_str(&value).map_err(server_error))
            .transpose()?;
        return state
            .user_space(profile_user_id, viewer_user_id)
            .await
            .map_err(server_error);
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = profile_user_id;
        let _ = viewer_user_id;
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
