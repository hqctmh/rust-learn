use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum NotificationType {
    FollowedUserPosted,
    PostCommented,
    CommentReplied,
    PostLiked,
    CommentLiked,
    Announcement,
    AdminMessage,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Notification {
    pub notification_id: Uuid,
    pub recipient_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum AnnouncementTarget {
    AllUsers,
    User(Uuid),
    Role(String),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AnnouncementRequest {
    pub title: String,
    pub body: String,
    pub target: AnnouncementTarget,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Announcement {
    pub announcement_id: Uuid,
    pub title: String,
    pub body: String,
    pub target: AnnouncementTarget,
    pub pinned: bool,
    pub published: bool,
    pub created_by: Uuid,
    pub created_at: OffsetDateTime,
}
