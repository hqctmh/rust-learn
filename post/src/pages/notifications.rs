use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{
    components::PageShell,
    domain::notifications::{Notification, NotificationCenter, NotificationType},
    page_data::{
        MarkAllPageNotificationsRead, MarkPageNotificationRead, fallback_notifications_page,
        load_notifications_page,
    },
};

#[component]
pub fn NotificationsPage() -> impl IntoView {
    let query = use_query_map();
    let fallback_query = query;
    let suspense_query = query;
    let center = Resource::new(
        move || query.read().get("session_id").unwrap_or_default(),
        |session_id| load_notifications_page(session_id),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <NotificationsCenter
                    center=fallback_notifications_page()
                    session_id=fallback_query.read().get("session_id").unwrap_or_default()
                />
            }>
                {move || Suspend::new(async move {
                    let center = center.await.unwrap_or_else(|_| fallback_notifications_page());
                    let session_id = suspense_query.read().get("session_id").unwrap_or_default();
                    view! { <NotificationsCenter center session_id/> }
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn NotificationsCenter(center: NotificationCenter, session_id: String) -> impl IntoView {
    let mark_read_action = ServerAction::<MarkPageNotificationRead>::new();
    let mark_all_action = ServerAction::<MarkAllPageNotificationsRead>::new();
    let mark_read_pending = mark_read_action.pending();
    let mark_all_pending = mark_all_action.pending();
    let mark_read_result = mark_read_action.value();
    let mark_all_result = mark_all_action.value();
    let initial_center = center.clone();
    let current_center = Memo::new(move |_| {
        if let Some(Ok(center)) = mark_all_result.get() {
            return center;
        }
        if let Some(Ok(center)) = mark_read_result.get() {
            return center;
        }
        initial_center.clone()
    });
    let mark_all_session_id = session_id.clone();
    let mark_all_disabled_session_id = session_id.clone();

    view! {
        <div class="notification-page">
            <section class="notification-hero">
                <div>
                    <div class="page-kicker">"消息中心"</div>
                    <h1>"通知与推送"</h1>
                    <p>"站内通知会记录评论、回复、点赞、关注用户发帖、公告和管理员消息。"</p>
                </div>
                <div class="unread-meter">
                    <strong>{move || current_center.get().unread_count}</strong>
                    <span>"未读通知"</span>
                </div>
            </section>

            <section class="notification-panel">
                <div class="notification-toolbar">
                    <div>
                        <h2>"历史通知"</h2>
                        <span>"支持单条已读、全部已读和 WebSocket 在线推送。"</span>
                    </div>
                    <ActionForm action=mark_all_action>
                        <input type="hidden" name="session_id" value=mark_all_session_id/>
                        <button
                            class="btn btn-outline"
                            type="submit"
                            disabled=move || mark_all_pending.get() || mark_all_disabled_session_id.is_empty()
                        >
                            "全部已读"
                        </button>
                    </ActionForm>
                </div>

                <div class="notification-action-feedback">
                    {move || {
                        if let Some(result) = mark_all_result.get() {
                            return match result {
                                Ok(center) => view! {
                                    <p class="success"><strong>"已读成功"</strong>{format!("剩余 {} 条未读", center.unread_count)}</p>
                                }.into_any(),
                                Err(error) => view! {
                                    <p class="error"><strong>"已读失败"</strong>{error.to_string()}</p>
                                }.into_any(),
                            };
                        }
                        if let Some(result) = mark_read_result.get() {
                            return match result {
                                Ok(center) => view! {
                                    <p class="success"><strong>"已读成功"</strong>{format!("剩余 {} 条未读", center.unread_count)}</p>
                                }.into_any(),
                                Err(error) => view! {
                                    <p class="error"><strong>"已读失败"</strong>{error.to_string()}</p>
                                }.into_any(),
                            };
                        }
                        ().into_any()
                    }}
                </div>

                <div class="notification-list">
                    {move || current_center
                        .get()
                        .items
                        .into_iter()
                        .map(|notification| view! {
                            <NotificationRow
                                notification
                                session_id=session_id.clone()
                                mark_read_action
                                mark_read_pending
                            />
                        })
                        .collect_view()}
                </div>
            </section>
        </div>
    }
}

#[component]
fn NotificationRow(
    notification: Notification,
    session_id: String,
    mark_read_action: ServerAction<MarkPageNotificationRead>,
    mark_read_pending: Memo<bool>,
) -> impl IntoView {
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
    let read_session_id = session_id.clone();
    let read_disabled_session_id = session_id.clone();

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
            <ActionForm action=mark_read_action>
                <input type="hidden" name="session_id" value=read_session_id/>
                <input
                    type="hidden"
                    name="notification_id"
                    value=notification.notification_id.to_string()
                />
                <button
                    class="btn btn-ghost btn-sm"
                    type="submit"
                    disabled=move || mark_read_pending.get() || read_disabled_session_id.is_empty() || !unread
                >
                    "标记已读"
                </button>
            </ActionForm>
        </article>
    }
}
