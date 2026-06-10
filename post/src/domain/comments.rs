use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CommentNode {
    pub comment_id: Uuid,
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub author_id: Uuid,
    pub author_name: String,
    pub content: String,
    pub deleted: bool,
    pub author_reply: bool,
    pub like_count: i64,
    pub created_at: OffsetDateTime,
    pub replies: Vec<CommentNode>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CommentInput {
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub content: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateCommentRequest {
    pub post_id: Uuid,
    pub parent_comment_id: Option<Uuid>,
    pub content: String,
}
