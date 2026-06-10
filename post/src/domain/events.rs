use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(tag = "event_type", content = "data")]
pub enum ForumEvent {
    #[serde(rename = "user.registered")]
    UserRegistered { user_id: Uuid, username: String },
    #[serde(rename = "user.followed")]
    UserFollowed {
        follower_id: Uuid,
        followee_id: Uuid,
        following: bool,
    },
    #[serde(rename = "post.created")]
    PostCreated {
        post_id: Uuid,
        author_id: Uuid,
        title: String,
    },
    #[serde(rename = "post.updated")]
    PostUpdated {
        post_id: Uuid,
        author_id: Uuid,
        title: String,
    },
    #[serde(rename = "post.deleted")]
    PostDeleted { post_id: Uuid, actor_id: Uuid },
    #[serde(rename = "post.liked")]
    PostLiked {
        post_id: Uuid,
        user_id: Uuid,
        active: bool,
    },
    #[serde(rename = "post.commented")]
    PostCommented {
        post_id: Uuid,
        comment_id: Uuid,
        author_id: Uuid,
    },
    #[serde(rename = "comment.replied")]
    CommentReplied {
        post_id: Uuid,
        comment_id: Uuid,
        parent_comment_id: Uuid,
        author_id: Uuid,
    },
    #[serde(rename = "announcement.published")]
    AnnouncementPublished {
        announcement_id: Uuid,
        title: String,
    },
    #[serde(rename = "notification.created")]
    NotificationCreated {
        notification_id: Uuid,
        recipient_id: Uuid,
    },
    #[serde(rename = "search.post.index")]
    SearchPostIndex {
        post_id: Uuid,
        title: String,
        body: String,
        tags: Vec<String>,
    },
    #[serde(rename = "search.post.delete")]
    SearchPostDelete { post_id: Uuid },
}

impl ForumEvent {
    pub fn subject(&self) -> &'static str {
        match self {
            Self::UserRegistered { .. } => "user.registered",
            Self::UserFollowed { .. } => "user.followed",
            Self::PostCreated { .. } => "post.created",
            Self::PostUpdated { .. } => "post.updated",
            Self::PostDeleted { .. } => "post.deleted",
            Self::PostLiked { .. } => "post.liked",
            Self::PostCommented { .. } => "post.commented",
            Self::CommentReplied { .. } => "comment.replied",
            Self::AnnouncementPublished { .. } => "announcement.published",
            Self::NotificationCreated { .. } => "notification.created",
            Self::SearchPostIndex { .. } => "search.post.index",
            Self::SearchPostDelete { .. } => "search.post.delete",
        }
    }
}
