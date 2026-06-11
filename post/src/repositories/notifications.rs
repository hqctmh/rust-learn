use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::notifications::{
    Notification, NotificationCenter, NotificationType, unread_count,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationRow {
    pub notification_id: Uuid,
    pub recipient_id: Uuid,
    pub actor_id: Option<Uuid>,
    pub notification_type: String,
    pub title: String,
    pub body: String,
    pub read_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

impl From<NotificationRow> for Notification {
    fn from(row: NotificationRow) -> Self {
        Self {
            notification_id: row.notification_id,
            recipient_id: row.recipient_id,
            actor_id: row.actor_id,
            notification_type: notification_type_from_str(&row.notification_type),
            title: row.title,
            body: row.body,
            read_at: row.read_at,
            created_at: row.created_at,
        }
    }
}

pub struct PostgresNotificationRepository;

impl PostgresNotificationRepository {
    pub async fn insert_notification(
        pool: &sqlx::PgPool,
        notification: &Notification,
    ) -> sqlx::Result<()> {
        let notification_type = notification_type_as_str(&notification.notification_type);
        sqlx::query!(
            r#"
insert into notifications (
    notification_id,
    recipient_id,
    actor_id,
    notification_type,
    title,
    body,
    read_at,
    created_at
)
values ($1, $2, $3, $4, $5, $6, $7, $8)
"#,
            notification.notification_id,
            notification.recipient_id,
            notification.actor_id,
            notification_type,
            notification.title,
            notification.body,
            notification.read_at,
            notification.created_at
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn followers_for_user(
        pool: &sqlx::PgPool,
        followee_id: Uuid,
    ) -> sqlx::Result<Vec<Uuid>> {
        let rows = sqlx::query!(
            r#"
select follower_id
from follows
where followee_id = $1
  and follower_id <> $1
"#,
            followee_id
        )
        .fetch_all(pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.follower_id).collect())
    }

    pub async fn notification_center(
        pool: &sqlx::PgPool,
        user_id: Uuid,
    ) -> sqlx::Result<Option<NotificationCenter>> {
        let user_exists = sqlx::query!(
            r#"
select exists (
    select 1
    from users
    where user_id = $1
      and status = 'active'
) as "exists!"
"#,
            user_id
        )
        .fetch_one(pool)
        .await?
        .exists;
        if !user_exists {
            return Ok(None);
        }

        let rows = sqlx::query_as!(
            NotificationRow,
            r#"
select
    notification_id,
    recipient_id,
    actor_id,
    notification_type,
    title,
    body,
    read_at,
    created_at
from notifications
where recipient_id = $1
order by created_at desc
"#,
            user_id
        )
        .fetch_all(pool)
        .await?;
        let items = rows.into_iter().map(Notification::from).collect::<Vec<_>>();

        Ok(Some(NotificationCenter {
            recipient_id: user_id,
            unread_count: unread_count(&items),
            items,
        }))
    }

    pub async fn mark_notification_read(
        pool: &sqlx::PgPool,
        user_id: Uuid,
        notification_id: Uuid,
    ) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update notifications
set read_at = coalesce(read_at, now())
where recipient_id = $1
  and notification_id = $2
"#,
            user_id,
            notification_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn mark_all_read(pool: &sqlx::PgPool, user_id: Uuid) -> sqlx::Result<u64> {
        let result = sqlx::query!(
            r#"
update notifications
set read_at = coalesce(read_at, now())
where recipient_id = $1
  and read_at is null
"#,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

fn notification_type_as_str(notification_type: &NotificationType) -> &'static str {
    match notification_type {
        NotificationType::FollowedUserPosted => "followed_user_posted",
        NotificationType::PostCommented => "post_commented",
        NotificationType::CommentReplied => "comment_replied",
        NotificationType::PostLiked => "post_liked",
        NotificationType::CommentLiked => "comment_liked",
        NotificationType::Announcement => "announcement",
        NotificationType::AdminMessage => "admin_message",
    }
}

fn notification_type_from_str(notification_type: &str) -> NotificationType {
    match notification_type {
        "followed_user_posted" => NotificationType::FollowedUserPosted,
        "comment_replied" => NotificationType::CommentReplied,
        "post_liked" => NotificationType::PostLiked,
        "comment_liked" => NotificationType::CommentLiked,
        "announcement" => NotificationType::Announcement,
        "admin_message" => NotificationType::AdminMessage,
        _ => NotificationType::PostCommented,
    }
}
