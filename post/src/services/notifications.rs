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
}
