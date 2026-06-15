use uuid::Uuid;

use crate::{
    domain::{
        comments::CommentNode,
        moderation::{
            ModerationCommentAction, ModerationCommentRow, ModerationPostAction, ModerationPostRow,
        },
        posts::{PostDetail, PostStatus},
    },
    error::ForumError,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommentModerationEffect {
    pub action: ModerationCommentAction,
    pub count_delta: i64,
}

pub struct ModerationService;

impl ModerationService {
    pub fn apply_post_status(
        post: &mut PostDetail,
        status: PostStatus,
        was_pinned: bool,
        was_recommended: bool,
    ) -> ModerationPostAction {
        post.status = status.clone();
        let pinned = was_pinned && status != PostStatus::Deleted;
        let recommended = was_recommended && status != PostStatus::Deleted;
        if status == PostStatus::Deleted {
            post.locked = false;
        }

        ModerationPostAction {
            post_id: post.summary.post_id,
            status,
            pinned,
            recommended,
            locked: post.locked,
        }
    }

    pub fn build_pin_action(
        post: &PostDetail,
        pinned: bool,
        recommended: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        if post.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能置顶".to_string()));
        }

        Ok(ModerationPostAction {
            post_id: post.summary.post_id,
            status: post.status.clone(),
            pinned,
            recommended,
            locked: post.locked,
        })
    }

    pub fn build_recommend_action(
        post: &PostDetail,
        recommended: bool,
        pinned: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        if post.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能推荐".to_string()));
        }

        Ok(ModerationPostAction {
            post_id: post.summary.post_id,
            status: post.status.clone(),
            pinned,
            recommended,
            locked: post.locked,
        })
    }

    pub fn build_lock_action(
        post: &mut PostDetail,
        locked: bool,
        pinned: bool,
        recommended: bool,
    ) -> Result<ModerationPostAction, ForumError> {
        if post.status == PostStatus::Deleted {
            return Err(ForumError::Conflict("已删除帖子不能锁定".to_string()));
        }

        post.locked = locked;
        Ok(ModerationPostAction {
            post_id: post.summary.post_id,
            status: post.status.clone(),
            pinned,
            recommended,
            locked,
        })
    }

    pub fn apply_comment_deleted(
        comment: &mut CommentNode,
        deleted: bool,
    ) -> CommentModerationEffect {
        let changed = comment.deleted != deleted;
        comment.deleted = deleted;
        let count_delta = if changed {
            if deleted { -1 } else { 1 }
        } else {
            0
        };

        CommentModerationEffect {
            action: ModerationCommentAction {
                comment_id: comment.comment_id,
                post_id: comment.post_id,
                deleted,
            },
            count_delta,
        }
    }

    pub fn apply_comment_count_delta(post: &mut PostDetail, delta: i64) {
        if delta < 0 {
            post.summary.comment_count = (post.summary.comment_count + delta).max(0);
        } else {
            post.summary.comment_count += delta;
        }
    }

    pub fn post_row(post: &PostDetail, pinned: bool, recommended: bool) -> ModerationPostRow {
        ModerationPostRow {
            post_id: post.summary.post_id,
            title: post.summary.title.clone(),
            author_name: post.summary.author_name.clone(),
            category_name: post.summary.category_name.clone(),
            status: post.status.clone(),
            pinned,
            recommended,
            locked: post.locked,
            comment_count: post.summary.comment_count,
            view_count: post.summary.view_count,
            updated_at: post.summary.published_at,
        }
    }

    pub fn flatten_comment_rows(
        post_id: Uuid,
        post_title: &str,
        comments: &[CommentNode],
    ) -> Vec<ModerationCommentRow> {
        let mut rows = Vec::new();
        for comment in comments {
            rows.push(ModerationCommentRow {
                comment_id: comment.comment_id,
                post_id,
                post_title: post_title.to_string(),
                author_name: comment.author_name.clone(),
                content: comment.content.clone(),
                deleted: comment.deleted,
                created_at: comment.created_at,
            });
            rows.extend(Self::flatten_comment_rows(
                post_id,
                post_title,
                &comment.replies,
            ));
        }
        rows
    }
}
