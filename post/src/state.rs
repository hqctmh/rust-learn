use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock},
};

use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        auth::{Session, SessionUser},
        comments::{CommentNode, CreateCommentRequest},
        posts::{CreatePostRequest, PostDetail, PostStatus, PostSummary},
        reactions::{FollowState, ToggleResult},
    },
    error::ForumError,
};

#[cfg(feature = "ssr")]
use sqlx::PgPool;

#[cfg(feature = "ssr")]
#[derive(Clone)]
pub struct AppState {
    pub db: Option<PgPool>,
    pub forum: ForumStore,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub database_url: String,
    pub redis_url: String,
    pub nats_url: String,
    pub rustfs_endpoint: String,
    pub elasticsearch_url: String,
}

impl RuntimeConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://post:post@localhost:5433/post".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6380".to_string()),
            nats_url: std::env::var("NATS_URL")
                .unwrap_or_else(|_| "nats://localhost:4222".to_string()),
            rustfs_endpoint: std::env::var("RUSTFS_ENDPOINT")
                .unwrap_or_else(|_| "http://localhost:9000".to_string()),
            elasticsearch_url: std::env::var("ELASTICSEARCH_URL")
                .unwrap_or_else(|_| "http://localhost:9200".to_string()),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ForumStore {
    inner: Arc<RwLock<ForumData>>,
}

#[derive(Debug)]
struct ForumData {
    users: HashMap<Uuid, SessionUser>,
    sessions: HashMap<Uuid, Session>,
    posts: HashMap<Uuid, PostDetail>,
    post_order: Vec<Uuid>,
    comments: HashMap<Uuid, Vec<CommentNode>>,
    liked_posts: HashSet<(Uuid, Uuid)>,
    favorited_posts: HashSet<(Uuid, Uuid)>,
    follows: HashSet<(Uuid, Uuid)>,
    next_id: u128,
}

impl ForumStore {
    pub fn seeded() -> Self {
        let author_id = Uuid::from_u128(1);
        let post_id = Uuid::from_u128(2);
        let now = OffsetDateTime::now_utc();

        let author = SessionUser {
            user_id: author_id,
            username: "mah".to_string(),
            nickname: "mah".to_string(),
            avatar_url: None,
            is_admin: true,
        };

        let summary = PostSummary {
            post_id,
            title: "Rust 异步任务的边界设计".to_string(),
            summary: "从论坛系统的通知链路拆分 Tokio 任务、事务和事件投递。".to_string(),
            author_id,
            author_name: author.nickname.clone(),
            author_avatar_url: author.avatar_url.clone(),
            category_name: Some("Rust".to_string()),
            tags: vec!["Leptos".to_string(), "SQLx".to_string()],
            view_count: 128,
            comment_count: 0,
            like_count: 19,
            favorite_count: 8,
            published_at: Some(now),
        };

        let detail = PostDetail {
            summary,
            markdown: "## Rust 异步任务的边界\n\n把通知、搜索索引和计数更新从请求链路中拆出去。".to_string(),
            sanitized_html: render_markdown_safe(
                "## Rust 异步任务的边界\n\n把通知、搜索索引和计数更新从请求链路中拆出去。",
            ),
            status: PostStatus::Published,
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        };

        let mut users = HashMap::new();
        users.insert(author_id, author);

        let mut posts = HashMap::new();
        posts.insert(post_id, detail);

        Self {
            inner: Arc::new(RwLock::new(ForumData {
                users,
                sessions: HashMap::new(),
                posts,
                post_order: vec![post_id],
                comments: HashMap::new(),
                liked_posts: HashSet::new(),
                favorited_posts: HashSet::new(),
                follows: HashSet::new(),
                next_id: 3,
            })),
        }
    }

    pub fn demo_user(&self) -> SessionUser {
        self.inner
            .read()
            .expect("forum store lock")
            .users
            .values()
            .next()
            .expect("seed user")
            .clone()
    }

    pub fn login(&self, username: &str, password: &str) -> Result<Session, ForumError> {
        let username = username.trim();
        if username.is_empty() || password.is_empty() {
            return Err(ForumError::Validation("用户名和密码不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        let user = data
            .users
            .values()
            .find(|user| user.username == username)
            .cloned()
            .unwrap_or_else(|| {
                let user_id = next_uuid(&mut data);
                let user = SessionUser {
                    user_id,
                    username: username.to_string(),
                    nickname: username.to_string(),
                    avatar_url: None,
                    is_admin: username == "admin",
                };
                data.users.insert(user_id, user.clone());
                user
            });

        let session_id = next_uuid(&mut data);
        let session = Session {
            session_id,
            user,
            expires_at: OffsetDateTime::now_utc() + Duration::days(7),
        };
        data.sessions.insert(session_id, session.clone());
        Ok(session)
    }

    pub fn list_posts(&self) -> Vec<PostSummary> {
        let data = self.inner.read().expect("forum store lock");
        data.post_order
            .iter()
            .filter_map(|post_id| data.posts.get(post_id))
            .filter(|detail| detail.status == PostStatus::Published)
            .map(|detail| detail.summary.clone())
            .collect()
    }

    pub fn post_detail(&self, post_id: Uuid) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        detail.summary.view_count += 1;
        Ok(detail.clone())
    }

    pub fn create_post(
        &self,
        author_id: Uuid,
        request: CreatePostRequest,
    ) -> Result<PostDetail, ForumError> {
        let title = request.title.trim();
        let markdown = request.markdown.trim();

        if title.is_empty() {
            return Err(ForumError::Validation("标题不能为空".to_string()));
        }
        if markdown.is_empty() {
            return Err(ForumError::Validation("正文不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        let author = data
            .users
            .get(&author_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        let post_id = next_uuid(&mut data);
        let published_at = request.publish.then(OffsetDateTime::now_utc);

        let summary = PostSummary {
            post_id,
            title: title.to_string(),
            summary: normalize_summary(&request.summary, markdown),
            author_id,
            author_name: author.nickname,
            author_avatar_url: author.avatar_url,
            category_name: request.category_name.filter(|value| !value.trim().is_empty()),
            tags: normalize_tags(request.tag_names),
            view_count: 0,
            comment_count: 0,
            like_count: 0,
            favorite_count: 0,
            published_at,
        };

        let detail = PostDetail {
            summary,
            markdown: markdown.to_string(),
            sanitized_html: render_markdown_safe(markdown),
            status: if request.publish {
                PostStatus::Published
            } else {
                PostStatus::Draft
            },
            liked_by_me: false,
            favorited_by_me: false,
            following_author: false,
        };

        data.post_order.insert(0, post_id);
        data.posts.insert(post_id, detail.clone());
        Ok(detail)
    }

    pub fn add_comment(
        &self,
        author_id: Uuid,
        request: CreateCommentRequest,
    ) -> Result<CommentNode, ForumError> {
        let content = request.content.trim();
        if content.is_empty() {
            return Err(ForumError::Validation("评论内容不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        let author = data
            .users
            .get(&author_id)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        let post_author_id = data
            .posts
            .get(&request.post_id)
            .ok_or(ForumError::NotFound)?
            .summary
            .author_id;
        let comment_id = next_uuid(&mut data);

        let comment = CommentNode {
            comment_id,
            post_id: request.post_id,
            parent_comment_id: request.parent_comment_id,
            author_id,
            author_name: author.nickname,
            content: content.to_string(),
            deleted: false,
            author_reply: author_id == post_author_id,
            like_count: 0,
            created_at: OffsetDateTime::now_utc(),
            replies: Vec::new(),
        };

        let comments = data.comments.entry(request.post_id).or_default();
        if let Some(parent_id) = request.parent_comment_id {
            append_reply(comments, parent_id, comment.clone())?;
        } else {
            comments.push(comment.clone());
        }

        if let Some(post) = data.posts.get_mut(&request.post_id) {
            post.summary.comment_count += 1;
        }
        Ok(comment)
    }

    pub fn comments_for_post(&self, post_id: Uuid) -> Result<Vec<CommentNode>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.posts.contains_key(&post_id) {
            return Err(ForumError::NotFound);
        }
        Ok(data.comments.get(&post_id).cloned().unwrap_or_default())
    }

    pub fn toggle_post_like(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        self.toggle_post_set(user_id, post_id, ReactionKind::Like)
    }

    pub fn toggle_post_favorite(
        &self,
        user_id: Uuid,
        post_id: Uuid,
    ) -> Result<ToggleResult, ForumError> {
        self.toggle_post_set(user_id, post_id, ReactionKind::Favorite)
    }

    pub fn follow_user(
        &self,
        follower_id: Uuid,
        followee_id: Uuid,
    ) -> Result<FollowState, ForumError> {
        if follower_id == followee_id {
            return Err(ForumError::Conflict("不能关注自己".to_string()));
        }

        let mut data = self.write_data()?;
        if !data.users.contains_key(&follower_id) || !data.users.contains_key(&followee_id) {
            return Err(ForumError::NotFound);
        }

        let key = (follower_id, followee_id);
        let following = if data.follows.contains(&key) {
            data.follows.remove(&key);
            false
        } else {
            data.follows.insert(key);
            true
        };

        Ok(FollowState {
            follower_id,
            followee_id,
            following,
        })
    }

    fn toggle_post_set(
        &self,
        user_id: Uuid,
        post_id: Uuid,
        kind: ReactionKind,
    ) -> Result<ToggleResult, ForumError> {
        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        if !data.posts.contains_key(&post_id) {
            return Err(ForumError::NotFound);
        }

        let key = (user_id, post_id);
        let active = match kind {
            ReactionKind::Like => toggle_set(&mut data.liked_posts, key),
            ReactionKind::Favorite => toggle_set(&mut data.favorited_posts, key),
        };

        let post = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        let count = match kind {
            ReactionKind::Like => apply_counter_delta(&mut post.summary.like_count, active),
            ReactionKind::Favorite => apply_counter_delta(&mut post.summary.favorite_count, active),
        };

        Ok(ToggleResult { active, count })
    }

    fn write_data(&self) -> Result<std::sync::RwLockWriteGuard<'_, ForumData>, ForumError> {
        self.inner.write().map_err(|_| ForumError::Internal)
    }
}

#[derive(Clone, Copy)]
enum ReactionKind {
    Like,
    Favorite,
}

fn next_uuid(data: &mut ForumData) -> Uuid {
    let id = data.next_id;
    data.next_id += 1;
    Uuid::from_u128(id)
}

fn normalize_summary(summary: &str, markdown: &str) -> String {
    let summary = summary.trim();
    if !summary.is_empty() {
        return summary.chars().take(180).collect();
    }

    markdown
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("暂未填写摘要")
        .trim_start_matches('#')
        .trim()
        .chars()
        .take(180)
        .collect()
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    tags.into_iter()
        .map(|tag| tag.trim().trim_start_matches('#').to_lowercase())
        .filter(|tag| !tag.is_empty())
        .fold(Vec::new(), |mut acc, tag| {
            if !acc.contains(&tag) {
                acc.push(tag);
            }
            acc
        })
}

fn render_markdown_safe(markdown: &str) -> String {
    markdown
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() {
                return None;
            }

            let escaped = escape_html(line);
            if let Some(heading) = escaped.strip_prefix("# ") {
                Some(format!("<h1>{heading}</h1>"))
            } else if let Some(heading) = escaped.strip_prefix("## ") {
                Some(format!("<h2>{heading}</h2>"))
            } else if let Some(quote) = escaped.strip_prefix("&gt; ") {
                Some(format!("<blockquote>{quote}</blockquote>"))
            } else {
                Some(format!("<p>{escaped}</p>"))
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_html(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn append_reply(
    comments: &mut [CommentNode],
    parent_id: Uuid,
    reply: CommentNode,
) -> Result<(), ForumError> {
    for comment in comments {
        if comment.comment_id == parent_id {
            comment.replies.push(reply);
            return Ok(());
        }
        if append_reply(&mut comment.replies, parent_id, reply.clone()).is_ok() {
            return Ok(());
        }
    }
    Err(ForumError::NotFound)
}

fn toggle_set(set: &mut HashSet<(Uuid, Uuid)>, key: (Uuid, Uuid)) -> bool {
    if set.contains(&key) {
        set.remove(&key);
        false
    } else {
        set.insert(key);
        true
    }
}

fn apply_counter_delta(count: &mut i64, active: bool) -> i64 {
    if active {
        *count += 1;
    } else {
        *count = (*count - 1).max(0);
    }
    *count
}
