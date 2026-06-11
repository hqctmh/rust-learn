use time::OffsetDateTime;
use uuid::Uuid;

use crate::{domain::comments::CommentNode, error::ForumError};

pub struct CommentService;

impl CommentService {
    pub fn build_comment(
        comment_id: Uuid,
        post_id: Uuid,
        parent_comment_id: Option<Uuid>,
        author_id: Uuid,
        author_name: &str,
        post_author_id: Uuid,
        content: &str,
        now: OffsetDateTime,
    ) -> Result<CommentNode, ForumError> {
        let content = content.trim();
        if content.is_empty() {
            return Err(ForumError::Validation("评论内容不能为空".to_string()));
        }

        Ok(CommentNode {
            comment_id,
            post_id,
            parent_comment_id,
            author_id,
            author_name: author_name.to_string(),
            content: content.to_string(),
            deleted: false,
            author_reply: author_id == post_author_id,
            like_count: 0,
            created_at: now,
            replies: Vec::new(),
        })
    }

    pub fn mask_deleted(mut comment: CommentNode) -> CommentNode {
        if comment.deleted {
            comment.content = "该评论已被删除".to_string();
        }
        comment.replies = comment
            .replies
            .into_iter()
            .map(Self::mask_deleted)
            .collect();
        comment
    }

    pub fn notification_body(content: &str) -> String {
        content.chars().take(80).collect()
    }
}
