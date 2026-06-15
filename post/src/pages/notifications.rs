use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
#[cfg(feature = "hydrate")]
use send_wrapper::SendWrapper;
#[cfg(feature = "hydrate")]
use wasm_bindgen::{JsCast, prelude::Closure};

#[cfg(feature = "hydrate")]
use crate::domain::notifications::NotificationPush;
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
    let realtime_recipient_id = center.recipient_id;
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

            <NotificationRealtimeClient recipient_id=realtime_recipient_id session_id=session_id.clone()/>

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
fn NotificationRealtimeClient(recipient_id: uuid::Uuid, session_id: String) -> impl IntoView {
    let status = RwSignal::new(if session_id.is_empty() {
        "实时推送等待登录".to_string()
    } else {
        "实时推送连接中".to_string()
    });
    let latest_push = RwSignal::new(None::<String>);
    #[cfg(not(feature = "hydrate"))]
    let _ = recipient_id;

    #[cfg(feature = "hydrate")]
    {
        let effect_session_id = session_id.clone();
        let effect_status = status;
        let effect_latest_push = latest_push;

        Effect::new(move |_| {
            if effect_session_id.is_empty() {
                effect_status.set("实时推送等待登录".to_string());
                return;
            }

            let Some(url) = notification_websocket_url(recipient_id) else {
                effect_status.set("实时推送连接失败".to_string());
                return;
            };

            let Ok(socket) = web_sys::WebSocket::new(&url) else {
                effect_status.set("实时推送连接失败".to_string());
                return;
            };
            effect_status.set("实时推送已连接".to_string());

            let message_status = effect_status;
            let message_latest_push = effect_latest_push;
            let onmessage =
                Closure::<dyn FnMut(web_sys::MessageEvent)>::wrap(Box::new(move |event| {
                    let Some(payload) = event.data().as_string() else {
                        message_status.set("实时推送解析失败".to_string());
                        return;
                    };
                    match serde_json::from_str::<NotificationPush>(&payload) {
                        Ok(push) => {
                            message_status.set("收到实时推送".to_string());
                            message_latest_push.set(Some(format!("{}：{}", push.title, push.body)));
                        }
                        Err(_) => message_status.set("实时推送解析失败".to_string()),
                    }
                }));
            socket.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
            onmessage.forget();

            let error_status = effect_status;
            let onerror = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_| {
                error_status.set("实时推送连接失败".to_string());
            }));
            socket.set_onerror(Some(onerror.as_ref().unchecked_ref()));
            onerror.forget();

            let cleanup_socket = SendWrapper::new(socket);
            Owner::on_cleanup(move || {
                let socket = cleanup_socket.take();
                let _ = socket.close();
            });
        });
    }

    view! {
        <section class="notification-realtime" aria-live="polite">
            <div>
                <h2>"实时通知"</h2>
                <p>{move || status.get()}</p>
            </div>
            <div class="notification-realtime-latest">
                {move || latest_push.get().unwrap_or_else(|| "等待新的推送消息".to_string())}
            </div>
        </section>
    }
}

#[cfg(feature = "hydrate")]
fn notification_websocket_url(recipient_id: uuid::Uuid) -> Option<String> {
    let location = web_sys::window()?.location();
    let protocol = location.protocol().ok()?;
    let host = location.host().ok()?;
    let scheme = if protocol == "https:" {
        "wss://"
    } else {
        "ws://"
    };
    Some(format!("{scheme}{host}/ws/notifications/{recipient_id}"))
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
