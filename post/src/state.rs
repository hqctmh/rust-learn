use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, RwLock, mpsc},
};

use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    domain::{
        auth::{RegisterRequest, Session, SessionUser, UpdateProfileRequest, UserStatus},
        comments::{CommentNode, CreateCommentRequest},
        events::ForumEvent,
        files::{FileUploadRequest, StoredFile},
        moderation::{
            AdminStats, AuditLogEntry, CreateReportRequest, ModerationAction, Report,
            ReportDecision, ReportStatus, ReportTarget,
        },
        notifications::{
            Announcement, AnnouncementRequest, AnnouncementTarget, Notification, NotificationType,
        },
        posts::{
            CreatePostRequest, PostDetail, PostStatus, PostSummary, SearchQuery, SearchResult,
            SearchSort, UpdatePostRequest,
        },
        rbac::{Permission, admin_permissions},
        reactions::{FollowState, ToggleResult},
        taxonomy::{Category, Tag, TagInput, TaxonomyInput},
    },
    error::ForumError,
    storage::{bucket_for_purpose, safe_filename},
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
    password_hashes: HashMap<Uuid, String>,
    user_permissions: HashMap<Uuid, Vec<Permission>>,
    sessions: HashMap<Uuid, Session>,
    posts: HashMap<Uuid, PostDetail>,
    post_order: Vec<Uuid>,
    comments: HashMap<Uuid, Vec<CommentNode>>,
    liked_posts: HashSet<(Uuid, Uuid)>,
    favorited_posts: HashSet<(Uuid, Uuid)>,
    follows: HashSet<(Uuid, Uuid)>,
    notifications: HashMap<Uuid, Vec<Notification>>,
    notification_subscribers: HashMap<Uuid, Vec<mpsc::Sender<Notification>>>,
    events: Vec<ForumEvent>,
    announcements: Vec<Announcement>,
    files: HashMap<Uuid, StoredFile>,
    reports: HashMap<Uuid, Report>,
    categories: HashMap<Uuid, Category>,
    tags: HashMap<Uuid, Tag>,
    audit_logs: Vec<AuditLogEntry>,
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
            bio: "论坛管理员".to_string(),
            status: UserStatus::Active,
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
            markdown: "## Rust 异步任务的边界\n\n把通知、搜索索引和计数更新从请求链路中拆出去。"
                .to_string(),
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
        let mut password_hashes = HashMap::new();
        password_hashes.insert(
            author_id,
            hash_password("demo-password").expect("seed password hash"),
        );
        let mut user_permissions = HashMap::new();
        user_permissions.insert(author_id, admin_permissions());

        let mut posts = HashMap::new();
        posts.insert(post_id, detail);

        Self {
            inner: Arc::new(RwLock::new(ForumData {
                users,
                password_hashes,
                user_permissions,
                sessions: HashMap::new(),
                posts,
                post_order: vec![post_id],
                comments: HashMap::new(),
                liked_posts: HashSet::new(),
                favorited_posts: HashSet::new(),
                follows: HashSet::new(),
                notifications: HashMap::new(),
                notification_subscribers: HashMap::new(),
                events: Vec::new(),
                announcements: Vec::new(),
                files: HashMap::new(),
                reports: HashMap::new(),
                categories: HashMap::new(),
                tags: HashMap::new(),
                audit_logs: Vec::new(),
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

        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let user = data
            .users
            .values()
            .find(|user| user.username == username)
            .cloned()
            .ok_or(ForumError::Unauthorized)?;
        if user.status == UserStatus::Disabled {
            return Err(ForumError::Forbidden);
        }
        let password_hash = data
            .password_hashes
            .get(&user.user_id)
            .ok_or(ForumError::Unauthorized)?
            .clone();
        drop(data);

        verify_password(password, &password_hash)?;
        let mut data = self.write_data()?;
        let session = create_session(&mut data, user);
        Ok(session)
    }

    pub fn register(&self, request: RegisterRequest) -> Result<Session, ForumError> {
        let username = request.username.trim();
        let password = request.password.trim();
        let nickname = request.nickname.trim();
        if username.is_empty() || password.is_empty() || nickname.is_empty() {
            return Err(ForumError::Validation(
                "用户名、密码和昵称不能为空".to_string(),
            ));
        }
        if password.len() < 6 {
            return Err(ForumError::Validation("密码至少需要 6 位".to_string()));
        }

        let mut data = self.write_data()?;
        if data.users.values().any(|user| user.username == username) {
            return Err(ForumError::Conflict("用户名已存在".to_string()));
        }

        let user_id = next_uuid(&mut data);
        let user = SessionUser {
            user_id,
            username: username.to_string(),
            nickname: nickname.to_string(),
            avatar_url: None,
            bio: String::new(),
            status: UserStatus::Active,
            is_admin: false,
        };
        let password_hash = hash_password(password)?;
        data.users.insert(user_id, user.clone());
        data.password_hashes.insert(user_id, password_hash);
        data.user_permissions.insert(user_id, Vec::new());
        push_event(
            &mut data,
            ForumEvent::UserRegistered {
                user_id,
                username: username.to_string(),
            },
        );

        let session = create_session(&mut data, user);
        Ok(session)
    }

    pub fn update_profile(
        &self,
        user_id: Uuid,
        request: UpdateProfileRequest,
    ) -> Result<SessionUser, ForumError> {
        let nickname = request.nickname.trim();
        if nickname.is_empty() {
            return Err(ForumError::Validation("昵称不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        let user = data
            .users
            .get_mut(&user_id)
            .ok_or(ForumError::Unauthorized)?;
        user.nickname = nickname.to_string();
        user.avatar_url = request.avatar_url.filter(|value| !value.trim().is_empty());
        user.bio = request.bio.trim().chars().take(240).collect();
        let updated = user.clone();

        for session in data
            .sessions
            .values_mut()
            .filter(|session| session.user.user_id == user_id)
        {
            session.user = updated.clone();
        }
        Ok(updated)
    }

    pub fn password_hash_for_user(&self, username: &str) -> Option<String> {
        let data = self.inner.read().ok()?;
        let user_id = data
            .users
            .values()
            .find(|user| user.username == username.trim())?
            .user_id;
        data.password_hashes.get(&user_id).cloned()
    }

    pub fn current_user(&self, session_id: Uuid) -> Result<SessionUser, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        let session = data
            .sessions
            .get(&session_id)
            .ok_or(ForumError::Unauthorized)?;
        if session.expires_at <= OffsetDateTime::now_utc() {
            return Err(ForumError::Unauthorized);
        }
        if session.user.status == UserStatus::Disabled {
            return Err(ForumError::Forbidden);
        }
        Ok(session.user.clone())
    }

    pub fn logout(&self, session_id: Uuid) -> Result<(), ForumError> {
        let mut data = self.write_data()?;
        if data.sessions.remove(&session_id).is_none() {
            return Err(ForumError::Unauthorized);
        }
        Ok(())
    }

    pub fn permissions_for_user(&self, user_id: Uuid) -> Result<Vec<Permission>, ForumError> {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }
        Ok(data
            .user_permissions
            .get(&user_id)
            .cloned()
            .unwrap_or_default())
    }

    pub fn require_permission(&self, user_id: Uuid, code: &str) -> Result<(), ForumError> {
        if self
            .permissions_for_user(user_id)?
            .iter()
            .any(|permission| permission.code == code)
        {
            Ok(())
        } else {
            Err(ForumError::Forbidden)
        }
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
            category_name: request
                .category_name
                .filter(|value| !value.trim().is_empty()),
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
        if request.publish {
            notify_followers_about_post(&mut data, author_id, &detail.summary);
            push_event(
                &mut data,
                ForumEvent::PostCreated {
                    post_id,
                    author_id,
                    title: detail.summary.title.clone(),
                },
            );
            push_event(
                &mut data,
                ForumEvent::SearchPostIndex {
                    post_id,
                    title: detail.summary.title.clone(),
                    body: detail.markdown.clone(),
                    tags: detail.summary.tags.clone(),
                },
            );
        }
        Ok(detail)
    }

    pub fn update_post(
        &self,
        actor_id: Uuid,
        post_id: Uuid,
        request: UpdatePostRequest,
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
        ensure_active_user(&data, actor_id)?;
        let author_id = data
            .posts
            .get(&post_id)
            .ok_or(ForumError::NotFound)?
            .summary
            .author_id;
        if actor_id != author_id && !has_permission(&data, actor_id, "post:update") {
            return Err(ForumError::Forbidden);
        }

        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        detail.summary.title = title.to_string();
        detail.summary.summary = normalize_summary(&request.summary, markdown);
        detail.summary.category_name = request
            .category_name
            .filter(|value| !value.trim().is_empty());
        detail.summary.tags = normalize_tags(request.tag_names);
        if request.publish && detail.summary.published_at.is_none() {
            detail.summary.published_at = Some(OffsetDateTime::now_utc());
        }
        detail.status = if request.publish {
            PostStatus::Published
        } else {
            PostStatus::Draft
        };
        detail.markdown = markdown.to_string();
        detail.sanitized_html = render_markdown_safe(markdown);
        let updated = detail.clone();

        if actor_id != author_id {
            push_audit_log(
                &mut data,
                actor_id,
                "post.update",
                "post",
                Some(post_id),
                None,
                Some(updated.summary.title.clone()),
            );
        }
        push_event(
            &mut data,
            ForumEvent::PostUpdated {
                post_id,
                author_id,
                title: updated.summary.title.clone(),
            },
        );
        match updated.status {
            PostStatus::Published => push_event(
                &mut data,
                ForumEvent::SearchPostIndex {
                    post_id,
                    title: updated.summary.title.clone(),
                    body: updated.markdown.clone(),
                    tags: updated.summary.tags.clone(),
                },
            ),
            PostStatus::Deleted | PostStatus::Offline => {
                push_event(&mut data, ForumEvent::SearchPostDelete { post_id })
            }
            PostStatus::Draft => {}
        }
        Ok(updated)
    }

    pub fn delete_post(&self, actor_id: Uuid, post_id: Uuid) -> Result<PostDetail, ForumError> {
        self.set_post_status_for_actor(actor_id, post_id, PostStatus::Deleted, false)
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
        if let Some(parent_comment_id) = comment.parent_comment_id {
            push_event(
                &mut data,
                ForumEvent::CommentReplied {
                    post_id: comment.post_id,
                    comment_id: comment.comment_id,
                    parent_comment_id,
                    author_id,
                },
            );
        } else {
            push_event(
                &mut data,
                ForumEvent::PostCommented {
                    post_id: comment.post_id,
                    comment_id: comment.comment_id,
                    author_id,
                },
            );
        }
        Ok(comment)
    }

    pub fn delete_comment(&self, actor_id: Uuid, comment_id: Uuid) -> Result<(), ForumError> {
        let mut data = self.write_data()?;
        ensure_active_user(&data, actor_id)?;
        let author_id = data
            .comments
            .values()
            .find_map(|comments| find_comment_author(comments, comment_id))
            .ok_or(ForumError::NotFound)?;
        if actor_id != author_id && !has_permission(&data, actor_id, "comment:delete") {
            return Err(ForumError::Forbidden);
        }

        for comments in data.comments.values_mut() {
            if mark_comment_deleted(comments, comment_id) {
                return Ok(());
            }
        }
        Err(ForumError::NotFound)
    }

    pub fn admin_delete_comment(&self, actor_id: Uuid, comment_id: Uuid) -> Result<(), ForumError> {
        self.require_permission(actor_id, "comment:delete")?;
        self.delete_comment(actor_id, comment_id)
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

        push_event(
            &mut data,
            ForumEvent::UserFollowed {
                follower_id,
                followee_id,
                following,
            },
        );
        Ok(FollowState {
            follower_id,
            followee_id,
            following,
        })
    }

    pub fn search_posts(&self, query: SearchQuery) -> SearchResult {
        let data = self.inner.read().expect("forum store lock");
        let keyword = query
            .keyword
            .as_ref()
            .map(|value| value.trim().to_lowercase());
        let category = query
            .category_name
            .as_ref()
            .map(|value| value.trim().to_lowercase());
        let tag = query.tag.as_ref().map(|value| normalize_tag(value));

        let mut items = data
            .post_order
            .iter()
            .filter_map(|post_id| data.posts.get(post_id))
            .filter(|detail| detail.status == PostStatus::Published)
            .filter(|detail| {
                keyword.as_ref().is_none_or(|keyword| {
                    let haystack = format!(
                        "{}\n{}\n{}",
                        detail.summary.title, detail.summary.summary, detail.markdown
                    )
                    .to_lowercase();
                    haystack.contains(keyword)
                })
            })
            .filter(|detail| {
                category.as_ref().is_none_or(|category| {
                    detail
                        .summary
                        .category_name
                        .as_ref()
                        .map(|value| value.to_lowercase() == *category)
                        .unwrap_or(false)
                })
            })
            .filter(|detail| {
                tag.as_ref()
                    .is_none_or(|tag| detail.summary.tags.iter().any(|item| item == tag))
            })
            .map(|detail| detail.summary.clone())
            .collect::<Vec<_>>();

        match query.sort {
            SearchSort::Latest => {
                items.sort_by_key(|summary| std::cmp::Reverse(summary.published_at))
            }
            SearchSort::Hot => items.sort_by_key(|summary| {
                std::cmp::Reverse(
                    summary.view_count + summary.comment_count * 3 + summary.like_count * 5,
                )
            }),
        }

        let page = query.page.max(1);
        let page_size = query.page_size.clamp(1, 100);
        let total = items.len();
        let start = (page - 1) * page_size;
        let items = items.into_iter().skip(start).take(page_size).collect();

        SearchResult { total, items }
    }

    pub fn list_notifications(&self, user_id: Uuid) -> Vec<Notification> {
        let data = self.inner.read().expect("forum store lock");
        data.notifications
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn subscribe_notifications(&self, user_id: Uuid) -> mpsc::Receiver<Notification> {
        let (sender, receiver) = mpsc::channel();
        self.inner
            .write()
            .expect("forum store lock")
            .notification_subscribers
            .entry(user_id)
            .or_default()
            .push(sender);
        receiver
    }

    pub fn mark_all_notifications_read(&self, user_id: Uuid) -> usize {
        let mut data = self.inner.write().expect("forum store lock");
        let now = OffsetDateTime::now_utc();
        let Some(notifications) = data.notifications.get_mut(&user_id) else {
            return 0;
        };

        let mut changed = 0;
        for notification in notifications {
            if notification.read_at.is_none() {
                notification.read_at = Some(now);
                changed += 1;
            }
        }
        changed
    }

    pub fn upload_file(
        &self,
        user_id: Uuid,
        request: FileUploadRequest,
    ) -> Result<StoredFile, ForumError> {
        let filename = safe_filename(&request.original_filename)?;
        if request.size_bytes == 0 || request.size_bytes > 5 * 1024 * 1024 {
            return Err(ForumError::Validation("文件大小超出限制".to_string()));
        }
        if !is_allowed_image_mime(&request.mime_type) {
            return Err(ForumError::Validation("不支持的文件类型".to_string()));
        }

        let mut data = self.write_data()?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }

        let bucket = bucket_for_purpose(&request.purpose).to_string();
        let object_key = format!("{user_id}/{}/{filename}", request.sha256);
        if let Some(existing) = data.files.values().find(|file| {
            file.uploaded_by == user_id
                && file.bucket == bucket
                && file.object_key == object_key
                && file.sha256 == request.sha256
        }) {
            return Ok(existing.clone());
        }

        let file_id = next_uuid(&mut data);
        let file = StoredFile {
            file_id,
            original_filename: filename,
            bucket: bucket.clone(),
            object_key: object_key.clone(),
            size_bytes: request.size_bytes,
            mime_type: request.mime_type,
            sha256: request.sha256,
            uploaded_by: user_id,
            public_url: format!("/files/{bucket}/{object_key}"),
            uploaded_at: OffsetDateTime::now_utc(),
        };

        data.files.insert(file_id, file.clone());
        Ok(file)
    }

    pub fn publish_announcement(
        &self,
        actor_id: Uuid,
        request: AnnouncementRequest,
    ) -> Result<Announcement, ForumError> {
        let title = request.title.trim();
        let body = request.body.trim();
        if title.is_empty() || body.is_empty() {
            return Err(ForumError::Validation("公告标题和内容不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        if !data.users.contains_key(&actor_id) {
            return Err(ForumError::Unauthorized);
        }

        let announcement = Announcement {
            announcement_id: next_uuid(&mut data),
            title: title.to_string(),
            body: body.to_string(),
            target: request.target,
            pinned: false,
            published: true,
            created_by: actor_id,
            created_at: OffsetDateTime::now_utc(),
        };

        let recipients = announcement_recipients(&data, &announcement.target);
        for recipient_id in recipients {
            push_notification(
                &mut data,
                recipient_id,
                Some(actor_id),
                NotificationType::Announcement,
                announcement.title.clone(),
                announcement.body.clone(),
            );
        }
        data.announcements.push(announcement.clone());
        push_event(
            &mut data,
            ForumEvent::AnnouncementPublished {
                announcement_id: announcement.announcement_id,
                title: announcement.title.clone(),
            },
        );
        push_audit_log(
            &mut data,
            actor_id,
            "announcement.publish",
            "announcement",
            Some(announcement.announcement_id),
            None,
            Some(announcement.title.clone()),
        );
        Ok(announcement)
    }

    pub fn create_report(&self, request: CreateReportRequest) -> Result<Report, ForumError> {
        if request.reason.trim().is_empty() {
            return Err(ForumError::Validation("举报原因不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        if !data.users.contains_key(&request.reporter_id) {
            return Err(ForumError::Unauthorized);
        }
        validate_report_target(&data, &request.target)?;

        let report_id = next_uuid(&mut data);
        let report = Report {
            report_id,
            reporter_id: request.reporter_id,
            target: request.target,
            reason: request.reason.trim().to_string(),
            note: request.note.filter(|value| !value.trim().is_empty()),
            status: ReportStatus::Open,
            handled_by: None,
            handled_at: None,
            created_at: OffsetDateTime::now_utc(),
        };
        data.reports.insert(report_id, report.clone());
        Ok(report)
    }

    pub fn resolve_report(
        &self,
        actor_id: Uuid,
        report_id: Uuid,
        decision: ReportDecision,
    ) -> Result<Report, ForumError> {
        let mut data = self.write_data()?;
        let actor = data.users.get(&actor_id).ok_or(ForumError::Unauthorized)?;
        if !actor.is_admin {
            return Err(ForumError::Forbidden);
        }

        let mut report = data
            .reports
            .get(&report_id)
            .cloned()
            .ok_or(ForumError::NotFound)?;

        let before = format!("{:?}", report.status);
        match decision {
            ReportDecision::Resolved { action } => {
                apply_moderation_action(&mut data, &report.target, action)?;
                report.status = ReportStatus::Resolved;
            }
            ReportDecision::Rejected => {
                report.status = ReportStatus::Rejected;
            }
        }
        report.handled_by = Some(actor_id);
        report.handled_at = Some(OffsetDateTime::now_utc());

        data.reports.insert(report_id, report.clone());
        push_audit_log(
            &mut data,
            actor_id,
            "report.resolve",
            "report",
            Some(report_id),
            Some(before),
            Some(format!("{:?}", report.status)),
        );
        Ok(report)
    }

    pub fn audit_logs(&self) -> Vec<AuditLogEntry> {
        self.inner
            .read()
            .expect("forum store lock")
            .audit_logs
            .clone()
    }

    pub fn event_outbox(&self) -> Vec<ForumEvent> {
        self.inner.read().expect("forum store lock").events.clone()
    }

    pub fn admin_stats(&self) -> AdminStats {
        let data = self.inner.read().expect("forum store lock");
        AdminStats {
            user_total: data.users.len(),
            post_total: data.posts.len(),
            comment_total: data
                .comments
                .values()
                .flat_map(|comments| comments.iter())
                .map(count_comment_tree)
                .sum(),
            like_total: data.liked_posts.len(),
            favorite_total: data.favorited_posts.len(),
            open_report_total: data
                .reports
                .values()
                .filter(|report| report.status == ReportStatus::Open)
                .count(),
            audit_log_total: data.audit_logs.len(),
            notification_total: data.notifications.values().map(Vec::len).sum(),
            file_total: data.files.len(),
        }
    }

    pub fn admin_users(&self) -> Vec<SessionUser> {
        let mut users = self
            .inner
            .read()
            .expect("forum store lock")
            .users
            .values()
            .cloned()
            .collect::<Vec<_>>();
        users.sort_by(|left, right| left.username.cmp(&right.username));
        users
    }

    pub fn set_user_disabled(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        disabled: bool,
    ) -> Result<SessionUser, ForumError> {
        self.require_permission(actor_id, "user:disable")?;
        let mut data = self.write_data()?;
        let status = if disabled {
            UserStatus::Disabled
        } else {
            UserStatus::Active
        };
        let user = data.users.get_mut(&user_id).ok_or(ForumError::NotFound)?;
        user.status = status.clone();
        let updated = user.clone();
        for session in data
            .sessions
            .values_mut()
            .filter(|session| session.user.user_id == user_id)
        {
            session.user.status = status.clone();
        }
        push_audit_log(
            &mut data,
            actor_id,
            if disabled {
                "user.disable"
            } else {
                "user.enable"
            },
            "user",
            Some(user_id),
            None,
            Some(format!("{:?}", updated.status)),
        );
        Ok(updated)
    }

    pub fn set_post_status(
        &self,
        actor_id: Uuid,
        post_id: Uuid,
        status: PostStatus,
    ) -> Result<PostDetail, ForumError> {
        self.set_post_status_for_actor(actor_id, post_id, status, true)
    }

    pub fn categories(&self) -> Vec<Category> {
        let mut categories = self
            .inner
            .read()
            .expect("forum store lock")
            .categories
            .values()
            .cloned()
            .collect::<Vec<_>>();
        categories.sort_by_key(|category| (category.sort_order, category.name.clone()));
        categories
    }

    pub fn tags(&self) -> Vec<Tag> {
        let mut tags = self
            .inner
            .read()
            .expect("forum store lock")
            .tags
            .values()
            .cloned()
            .collect::<Vec<_>>();
        tags.sort_by(|left, right| left.name.cmp(&right.name));
        tags
    }

    pub fn create_category(
        &self,
        actor_id: Uuid,
        request: TaxonomyInput,
    ) -> Result<Category, ForumError> {
        self.require_permission(actor_id, "category:create")?;
        let name = request.name.trim();
        let slug = normalize_slug(&request.slug);
        if name.is_empty() || slug.is_empty() {
            return Err(ForumError::Validation("分类名称和标识不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        if data
            .categories
            .values()
            .any(|category| category.name == name || category.slug == slug)
        {
            return Err(ForumError::Conflict("分类已存在".to_string()));
        }
        let category = Category {
            category_id: next_uuid(&mut data),
            name: name.to_string(),
            slug,
            sort_order: request.sort_order,
            created_at: OffsetDateTime::now_utc(),
        };
        data.categories
            .insert(category.category_id, category.clone());
        push_audit_log(
            &mut data,
            actor_id,
            "category.create",
            "category",
            Some(category.category_id),
            None,
            Some(category.name.clone()),
        );
        Ok(category)
    }

    pub fn create_tag(&self, actor_id: Uuid, request: TagInput) -> Result<Tag, ForumError> {
        self.require_permission(actor_id, "tag:create")?;
        let name = request.name.trim();
        let slug = normalize_slug(&request.slug);
        if name.is_empty() || slug.is_empty() {
            return Err(ForumError::Validation("标签名称和标识不能为空".to_string()));
        }

        let mut data = self.write_data()?;
        if data
            .tags
            .values()
            .any(|tag| tag.name == name || tag.slug == slug)
        {
            return Err(ForumError::Conflict("标签已存在".to_string()));
        }
        let tag = Tag {
            tag_id: next_uuid(&mut data),
            name: name.to_string(),
            slug,
            created_at: OffsetDateTime::now_utc(),
        };
        data.tags.insert(tag.tag_id, tag.clone());
        push_audit_log(
            &mut data,
            actor_id,
            "tag.create",
            "tag",
            Some(tag.tag_id),
            None,
            Some(tag.name.clone()),
        );
        Ok(tag)
    }

    fn set_post_status_for_actor(
        &self,
        actor_id: Uuid,
        post_id: Uuid,
        status: PostStatus,
        require_admin: bool,
    ) -> Result<PostDetail, ForumError> {
        let mut data = self.write_data()?;
        ensure_active_user(&data, actor_id)?;
        let author_id = data
            .posts
            .get(&post_id)
            .ok_or(ForumError::NotFound)?
            .summary
            .author_id;
        if require_admin {
            if !has_permission(&data, actor_id, "post:update") {
                return Err(ForumError::Forbidden);
            }
        } else if actor_id != author_id && !has_permission(&data, actor_id, "post:delete") {
            return Err(ForumError::Forbidden);
        }

        let detail = data.posts.get_mut(&post_id).ok_or(ForumError::NotFound)?;
        detail.status = status.clone();
        let updated = detail.clone();
        push_audit_log(
            &mut data,
            actor_id,
            match status {
                PostStatus::Draft => "post.draft",
                PostStatus::Published => "post.publish",
                PostStatus::Offline => "post.offline",
                PostStatus::Deleted => "post.delete",
            },
            "post",
            Some(post_id),
            None,
            Some(format!("{:?}", updated.status)),
        );
        match updated.status {
            PostStatus::Deleted => {
                push_event(&mut data, ForumEvent::PostDeleted { post_id, actor_id });
                push_event(&mut data, ForumEvent::SearchPostDelete { post_id });
            }
            PostStatus::Offline => {
                push_event(&mut data, ForumEvent::SearchPostDelete { post_id });
            }
            PostStatus::Published => {
                push_event(
                    &mut data,
                    ForumEvent::SearchPostIndex {
                        post_id,
                        title: updated.summary.title.clone(),
                        body: updated.markdown.clone(),
                        tags: updated.summary.tags.clone(),
                    },
                );
            }
            PostStatus::Draft => {}
        }
        Ok(updated)
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

        if matches!(kind, ReactionKind::Like) {
            push_event(
                &mut data,
                ForumEvent::PostLiked {
                    post_id,
                    user_id,
                    active,
                },
            );
        }
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

fn create_session(data: &mut ForumData, user: SessionUser) -> Session {
    let session_id = next_uuid(data);
    let session = Session {
        session_id,
        user,
        expires_at: OffsetDateTime::now_utc() + Duration::days(7),
    };
    data.sessions.insert(session_id, session.clone());
    session
}

fn hash_password(password: &str) -> Result<String, ForumError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| ForumError::Internal)
}

fn verify_password(password: &str, password_hash: &str) -> Result<(), ForumError> {
    let parsed_hash = PasswordHash::new(password_hash).map_err(|_| ForumError::Unauthorized)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| ForumError::Unauthorized)
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
        .map(|tag| normalize_tag(&tag))
        .filter(|tag| !tag.is_empty())
        .fold(Vec::new(), |mut acc, tag| {
            if !acc.contains(&tag) {
                acc.push(tag);
            }
            acc
        })
}

fn normalize_tag(tag: &str) -> String {
    tag.trim().trim_start_matches('#').to_lowercase()
}

fn normalize_slug(slug: &str) -> String {
    slug.trim()
        .trim_matches('-')
        .to_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-')
        .collect()
}

fn ensure_active_user(data: &ForumData, user_id: Uuid) -> Result<(), ForumError> {
    let user = data.users.get(&user_id).ok_or(ForumError::Unauthorized)?;
    if user.status == UserStatus::Disabled {
        Err(ForumError::Forbidden)
    } else {
        Ok(())
    }
}

fn has_permission(data: &ForumData, user_id: Uuid, code: &str) -> bool {
    data.user_permissions
        .get(&user_id)
        .map(|permissions| permissions.iter().any(|permission| permission.code == code))
        .unwrap_or(false)
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

fn notify_followers_about_post(data: &mut ForumData, author_id: Uuid, post: &PostSummary) {
    let recipients = data
        .follows
        .iter()
        .filter_map(|(follower_id, followee_id)| {
            (*followee_id == author_id).then_some(*follower_id)
        })
        .collect::<Vec<_>>();

    for recipient_id in recipients {
        push_notification(
            data,
            recipient_id,
            Some(author_id),
            NotificationType::FollowedUserPosted,
            post.title.clone(),
            format!("{} 发布了新帖子", post.author_name),
        );
    }
}

fn push_notification(
    data: &mut ForumData,
    recipient_id: Uuid,
    actor_id: Option<Uuid>,
    notification_type: NotificationType,
    title: String,
    body: String,
) {
    let notification = Notification {
        notification_id: next_uuid(data),
        recipient_id,
        actor_id,
        notification_type,
        title,
        body,
        read_at: None,
        created_at: OffsetDateTime::now_utc(),
    };
    data.notifications
        .entry(recipient_id)
        .or_default()
        .insert(0, notification);
    let notification_id = data
        .notifications
        .get(&recipient_id)
        .and_then(|items| items.first())
        .expect("notification inserted")
        .notification_id;
    push_event(
        data,
        ForumEvent::NotificationCreated {
            notification_id,
            recipient_id,
        },
    );
    if let Some(subscribers) = data.notification_subscribers.get_mut(&recipient_id) {
        let latest = data
            .notifications
            .get(&recipient_id)
            .and_then(|items| items.first())
            .expect("notification inserted")
            .clone();
        subscribers.retain(|sender| sender.send(latest.clone()).is_ok());
    }
}

fn push_event(data: &mut ForumData, event: ForumEvent) {
    data.events.push(event);
}

fn announcement_recipients(data: &ForumData, target: &AnnouncementTarget) -> Vec<Uuid> {
    match target {
        AnnouncementTarget::AllUsers => data.users.keys().copied().collect(),
        AnnouncementTarget::User(user_id) => data
            .users
            .contains_key(user_id)
            .then_some(*user_id)
            .into_iter()
            .collect(),
        AnnouncementTarget::Role(role) if role == "admin" => data
            .users
            .values()
            .filter(|user| user.is_admin)
            .map(|user| user.user_id)
            .collect(),
        AnnouncementTarget::Role(_) => Vec::new(),
    }
}

fn is_allowed_image_mime(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/gif" | "image/webp"
    )
}

fn validate_report_target(data: &ForumData, target: &ReportTarget) -> Result<(), ForumError> {
    match target {
        ReportTarget::Post(post_id) if data.posts.contains_key(post_id) => Ok(()),
        ReportTarget::User(user_id) if data.users.contains_key(user_id) => Ok(()),
        ReportTarget::Comment(comment_id)
            if data
                .comments
                .values()
                .any(|comments| contains_comment(comments, *comment_id)) =>
        {
            Ok(())
        }
        _ => Err(ForumError::NotFound),
    }
}

fn contains_comment(comments: &[CommentNode], comment_id: Uuid) -> bool {
    comments.iter().any(|comment| {
        comment.comment_id == comment_id || contains_comment(&comment.replies, comment_id)
    })
}

fn find_comment_author(comments: &[CommentNode], comment_id: Uuid) -> Option<Uuid> {
    comments.iter().find_map(|comment| {
        if comment.comment_id == comment_id {
            Some(comment.author_id)
        } else {
            find_comment_author(&comment.replies, comment_id)
        }
    })
}

fn apply_moderation_action(
    data: &mut ForumData,
    target: &ReportTarget,
    action: ModerationAction,
) -> Result<(), ForumError> {
    match (target, action) {
        (ReportTarget::Post(post_id), ModerationAction::TakePostOffline) => {
            let post = data.posts.get_mut(post_id).ok_or(ForumError::NotFound)?;
            post.status = PostStatus::Offline;
            Ok(())
        }
        (ReportTarget::Comment(comment_id), ModerationAction::DeleteComment) => {
            for comments in data.comments.values_mut() {
                if mark_comment_deleted(comments, *comment_id) {
                    return Ok(());
                }
            }
            Err(ForumError::NotFound)
        }
        (ReportTarget::User(user_id), ModerationAction::DisableUser) => {
            data.users.get(user_id).ok_or(ForumError::NotFound)?;
            Ok(())
        }
        (_, ModerationAction::NoAction) => Ok(()),
        _ => Ok(()),
    }
}

fn mark_comment_deleted(comments: &mut [CommentNode], comment_id: Uuid) -> bool {
    for comment in comments {
        if comment.comment_id == comment_id {
            comment.deleted = true;
            comment.content = "该评论已被删除".to_string();
            return true;
        }
        if mark_comment_deleted(&mut comment.replies, comment_id) {
            return true;
        }
    }
    false
}

fn push_audit_log(
    data: &mut ForumData,
    actor_id: Uuid,
    action: &str,
    target_type: &str,
    target_id: Option<Uuid>,
    before: Option<String>,
    after: Option<String>,
) {
    let audit_id = next_uuid(data);
    data.audit_logs.push(AuditLogEntry {
        audit_id,
        actor_id,
        action: action.to_string(),
        target_type: target_type.to_string(),
        target_id,
        before,
        after,
        ip: None,
        user_agent: None,
        created_at: OffsetDateTime::now_utc(),
    });
}

fn count_comment_tree(comment: &CommentNode) -> usize {
    1 + comment
        .replies
        .iter()
        .map(count_comment_tree)
        .sum::<usize>()
}
