use uuid::Uuid;

use crate::domain::notifications::{Notification, NotificationPush};

pub struct NotificationPushService;

impl NotificationPushService {
    pub fn build_pending_push(
        push_id: Uuid,
        online_connections: usize,
        notification: Notification,
    ) -> Option<NotificationPush> {
        (online_connections > 0).then(|| (push_id, notification).into())
    }

    #[cfg(feature = "ssr")]
    pub fn websocket_payload(push: &NotificationPush) -> Result<String, serde_json::Error> {
        Ok(serde_json::json!({
            "type": "notification",
            "push_id": push.push_id,
            "notification_id": push.notification_id,
            "recipient_id": push.recipient_id,
            "actor_id": push.actor_id,
            "notification_type": push.notification_type,
            "title": push.title,
            "body": push.body,
            "created_at": push.created_at,
        })
        .to_string())
    }

    #[cfg(feature = "ssr")]
    pub fn ack_message_to_push_id(message: &str) -> Option<Uuid> {
        let trimmed = message.trim();
        if let Ok(push_id) = Uuid::parse_str(trimmed) {
            return Some(push_id);
        }

        let value = serde_json::from_str::<serde_json::Value>(trimmed).ok()?;
        if value.get("type").and_then(serde_json::Value::as_str) != Some("ack") {
            return None;
        }
        value
            .get("push_id")
            .and_then(serde_json::Value::as_str)
            .and_then(|push_id| Uuid::parse_str(push_id).ok())
    }
}
