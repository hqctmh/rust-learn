use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum PostStatus {
    Draft,
    Published,
    Offline,
    Deleted,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostSummary {
    pub post_id: Uuid,
    pub title: String,
    pub summary: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_avatar_url: Option<String>,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub view_count: i64,
    pub comment_count: i64,
    pub like_count: i64,
    pub favorite_count: i64,
    pub published_at: Option<OffsetDateTime>,
}

impl PostSummary {
    pub fn sample() -> Self {
        Self {
            post_id: Uuid::nil(),
            title: "Rust 异步任务的边界设计".to_string(),
            summary: "从论坛系统的通知链路拆分 Tokio 任务、事务和事件投递。".to_string(),
            author_id: Uuid::nil(),
            author_name: "mah".to_string(),
            author_avatar_url: None,
            category_name: Some("Rust".to_string()),
            tags: vec!["Leptos".to_string(), "SQLx".to_string()],
            view_count: 128,
            comment_count: 6,
            like_count: 19,
            favorite_count: 8,
            published_at: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostDetail {
    pub summary: PostSummary,
    pub markdown: String,
    pub sanitized_html: String,
    pub status: PostStatus,
    pub liked_by_me: bool,
    pub favorited_by_me: bool,
    pub following_author: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PostEditorDraft {
    pub title: String,
    pub markdown: String,
    pub summary: String,
    pub cover_url: Option<String>,
    pub category_id: Option<Uuid>,
    pub tag_names: Vec<String>,
    pub publish: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreatePostRequest {
    pub title: String,
    pub markdown: String,
    pub summary: String,
    pub category_name: Option<String>,
    pub tag_names: Vec<String>,
    pub publish: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdatePostRequest {
    pub title: String,
    pub markdown: String,
    pub summary: String,
    pub category_name: Option<String>,
    pub tag_names: Vec<String>,
    pub publish: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AutosaveDraftRequest {
    pub post_id: Option<Uuid>,
    pub title: String,
    pub markdown: String,
    pub summary: String,
    pub category_name: Option<String>,
    pub tag_names: Vec<String>,
}
