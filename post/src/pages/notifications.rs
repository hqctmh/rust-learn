use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::notifications::{Notification, NotificationType, notification_demo_center},
};

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let center = notification_demo_center();
    let unread_count = center.unread_count;
    let notifications = center.items.clone();

    view! {
        <PageShell>
            <div class="notification-page">
                <section class="notification-hero">
                    <div>
                        <div class="page-kicker">"消息中心"</div>
                        <h1>"通知与推送"</h1>
                        <p>"站内通知会记录评论、回复、点赞、关注用户发帖、公告和管理员消息。"</p>
                    </div>
                    <div class="unread-meter">
                        <strong>{unread_count}</strong>
                        <span>"未读通知"</span>
                    </div>
                </section>

                <section class="notification-panel">
                    <div class="notification-toolbar">
                        <div>
                            <h2>"历史通知"</h2>
                            <span>"支持单条已读、全部已读和 WebSocket 在线推送。"</span>
                        </div>
                        <button class="btn btn-outline">"全部已读"</button>
                    </div>

                    <div class="notification-list">
                        {notifications.into_iter().map(|notification| view! {
                            <NotificationRow notification/>
                        }).collect_view()}
                    </div>
                </section>
            </div>
        </PageShell>
    }
}

#[component]
fn NotificationRow(notification: Notification) -> impl IntoView {
    let unread = notification.read_at.is_none();
    let type_label = match notification.notification_type {
        NotificationType::FollowedUserPosted => "关注动态",
        NotificationType::PostCommented => "帖子评论",
        NotificationType::CommentReplied => "评论回复",
        NotificationType::PostLiked => "帖子点赞",
        NotificationType::CommentLiked => "评论点赞",
        NotificationType::Announcement => "系统公告",
        NotificationType::AdminMessage => "管理员通知",
    };

    view! {
        <article class=if unread { "notification-row unread" } else { "notification-row" }>
            <div class="notification-dot" aria-hidden="true"></div>
            <div>
                <div class="notification-title-line">
                    <span>{type_label}</span>
                    <h3>{notification.title}</h3>
                </div>
                <p>{notification.body}</p>
            </div>
            <button class="btn btn-ghost btn-sm">"标记已读"</button>
        </article>
    }
}
