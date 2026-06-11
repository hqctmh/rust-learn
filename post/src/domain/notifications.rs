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
pub struct NotificationCenter {
    pub recipient_id: Uuid,
    pub unread_count: usize,
    pub items: Vec<Notification>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NotificationPush {
    pub push_id: Uuid,
    pub notification_id: Uuid,
    pub recipient_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub notification_type: NotificationType,
    pub title: String,
    pub body: String,
    pub created_at: OffsetDateTime,
}

impl From<(Uuid, Notification)> for NotificationPush {
    fn from((push_id, notification): (Uuid, Notification)) -> Self {
        Self {
            push_id,
            notification_id: notification.notification_id,
            recipient_id: notification.recipient_id,
            actor_id: notification.actor_id,
            notification_type: notification.notification_type,
            title: notification.title,
            body: notification.body,
            created_at: notification.created_at,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NotificationConnectionStats {
    pub user_id: Uuid,
    pub online_connections: usize,
    pub pending_push_count: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NotificationReadRequest {
    pub user_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct NotificationPushAckRequest {
    pub user_id: Option<Uuid>,
    pub push_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnreadCount {
    pub unread_count: usize,
}

pub fn notification_demo_center() -> NotificationCenter {
    let recipient_id = Uuid::from_u128(1);
    let now = OffsetDateTime::now_utc();
    let items = vec![
        Notification {
            notification_id: Uuid::from_u128(9001),
            recipient_id,
            actor_id: Some(Uuid::from_u128(3)),
            notification_type: NotificationType::PostCommented,
            title: "wangxy 评论了你的帖子".to_string(),
            body: "在 server function 中使用 SQLx 事务的最佳实践有新评论。".to_string(),
            read_at: None,
            created_at: now,
        },
        Notification {
            notification_id: Uuid::from_u128(9002),
            recipient_id,
            actor_id: None,
            notification_type: NotificationType::Announcement,
            title: "论坛升级与搜索增强说明".to_string(),
            body: "搜索结果页已支持关键词高亮和筛选。".to_string(),
            read_at: Some(now),
            created_at: now,
        },
    ];

    NotificationCenter {
        recipient_id,
        unread_count: unread_count(&items),
        items,
    }
}

pub fn unread_count(items: &[Notification]) -> usize {
    items
        .iter()
        .filter(|notification| notification.read_at.is_none())
        .count()
}
