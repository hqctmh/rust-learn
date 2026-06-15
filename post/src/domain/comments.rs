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
pub struct CommentPageQuery {
    pub page: usize,
    pub page_size: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CommentPage {
    pub comments: Vec<CommentNode>,
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
}

impl Default for CommentPageQuery {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
        }
    }
}

impl CommentPageQuery {
    pub fn normalized(mut self) -> Self {
        if self.page == 0 {
            self.page = 1;
        }
        if self.page_size == 0 {
            self.page_size = 20;
        }
        self.page_size = self.page_size.min(100);
        self
    }
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
