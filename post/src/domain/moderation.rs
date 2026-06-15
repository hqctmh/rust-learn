use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::posts::PostStatus;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationPostRow {
    pub post_id: Uuid,
    pub title: String,
    pub author_name: String,
    pub category_name: Option<String>,
    pub status: PostStatus,
    pub pinned: bool,
    pub recommended: bool,
    pub locked: bool,
    pub comment_count: i64,
    pub view_count: i64,
    pub updated_at: Option<OffsetDateTime>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationCommentRow {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub post_title: String,
    pub author_name: String,
    pub content: String,
    pub deleted: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationPostAction {
    pub post_id: Uuid,
    pub status: PostStatus,
    pub pinned: bool,
    pub recommended: bool,
    pub locked: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ModerationCommentAction {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub deleted: bool,
}
