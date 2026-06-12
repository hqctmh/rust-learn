use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{
    components::PageShell,
    domain::admin::{
        AdminAnnouncementRow, AdminCategoryRow, AdminCommentRow, AdminDashboard, AdminPostRow,
        AdminReportRow, AdminStat, AdminTagRow, AdminUserRow,
    },
    page_data::{
        DeleteAdminComment, DeleteAdminPost, DisableAdminUser, EnableAdminUser, HandleAdminReport,
        PinAdminPost, PublishAdminAnnouncement, RecoverAdminComment, RejectAdminReport,
        RestoreAdminPost, TakeDownAdminPost, UnpinAdminPost, WithdrawAdminAnnouncement,
        fallback_admin_dashboard, load_admin_dashboard,
    },
};

#[component]
pub fn AdminPage() -> impl IntoView {
    let query = use_query_map();
    let fallback_query = query;
    let suspense_query = query;
    let dashboard = Resource::new(
        move || query.read().get("session_id").unwrap_or_default(),
        |session_id| load_admin_dashboard(session_id),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <AdminDashboardView
                    dashboard=fallback_admin_dashboard()
                    session_id=fallback_query.read().get("session_id").unwrap_or_default()
                />
            }>
                {move || Suspend::new(async move {
                    let dashboard = dashboard.await.unwrap_or_else(|_| fallback_admin_dashboard());
                    let session_id = suspense_query.read().get("session_id").unwrap_or_default();
                    view! { <AdminDashboardView dashboard session_id/> }
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn AdminDashboardView(dashboard: AdminDashboard, session_id: String) -> impl IntoView {
    let disable_user_action = ServerAction::<DisableAdminUser>::new();
    let enable_user_action = ServerAction::<EnableAdminUser>::new();
    let take_down_post_action = ServerAction::<TakeDownAdminPost>::new();
    let restore_post_action = ServerAction::<RestoreAdminPost>::new();
    let delete_post_action = ServerAction::<DeleteAdminPost>::new();
    let pin_post_action = ServerAction::<PinAdminPost>::new();
    let unpin_post_action = ServerAction::<UnpinAdminPost>::new();
    let delete_comment_action = ServerAction::<DeleteAdminComment>::new();
    let recover_comment_action = ServerAction::<RecoverAdminComment>::new();
    let handle_report_action = ServerAction::<HandleAdminReport>::new();
    let reject_report_action = ServerAction::<RejectAdminReport>::new();
    let publish_announcement_action = ServerAction::<PublishAdminAnnouncement>::new();
    let withdraw_announcement_action = ServerAction::<WithdrawAdminAnnouncement>::new();
    let disable_user_pending = disable_user_action.pending();
    let enable_user_pending = enable_user_action.pending();
    let take_down_post_pending = take_down_post_action.pending();
    let restore_post_pending = restore_post_action.pending();
    let delete_post_pending = delete_post_action.pending();
    let pin_post_pending = pin_post_action.pending();
    let unpin_post_pending = unpin_post_action.pending();
    let delete_comment_pending = delete_comment_action.pending();
    let recover_comment_pending = recover_comment_action.pending();
    let handle_report_pending = handle_report_action.pending();
    let reject_report_pending = reject_report_action.pending();
    let publish_announcement_pending = publish_announcement_action.pending();
    let withdraw_announcement_pending = withdraw_announcement_action.pending();
    let disable_user_result = disable_user_action.value();
    let enable_user_result = enable_user_action.value();
    let take_down_post_result = take_down_post_action.value();
    let restore_post_result = restore_post_action.value();
    let delete_post_result = delete_post_action.value();
    let pin_post_result = pin_post_action.value();
    let unpin_post_result = unpin_post_action.value();
    let delete_comment_result = delete_comment_action.value();
    let recover_comment_result = recover_comment_action.value();
    let handle_report_result = handle_report_action.value();
    let reject_report_result = reject_report_action.value();
    let publish_announcement_result = publish_announcement_action.value();
    let withdraw_announcement_result = withdraw_announcement_action.value();
    let initial_dashboard = dashboard.clone();
    let current_dashboard = Memo::new(move |_| {
        if let Some(Ok(dashboard)) = withdraw_announcement_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = publish_announcement_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = reject_report_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = handle_report_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = recover_comment_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = delete_comment_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = unpin_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = pin_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = delete_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = restore_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = take_down_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = enable_user_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = disable_user_result.get() {
            return dashboard;
        }
        initial_dashboard.clone()
    });
    let menu = dashboard.menu.clone();
    let stats = dashboard.stats.clone();
    let categories = dashboard.categories.clone();
    let tags = dashboard.tags.clone();
    let queues = dashboard.governance_queue.clone();
    let audit_entries = dashboard.audit_entries.clone();
    let user_table_session_id = session_id.clone();
    let post_table_session_id = session_id.clone();
    let comment_table_session_id = session_id.clone();
    let announcement_table_session_id = session_id.clone();
    let report_table_session_id = session_id.clone();

    view! {
        <div class="admin-layout">
            <aside class="admin-sidebar">
                <div class="page-kicker">"RBAC 管理后台"</div>
                <ul class="menu">
                    {menu.into_iter().enumerate().map(|(index, item)| view! {
                        <li><a class=if index == 0 { "active" } else { "" }>{item.label}</a></li>
                    }).collect_view()}
                </ul>
            </aside>
            <section class="admin-main">
                    <div class="section-heading">
                        <div>
                            <div class="page-kicker">"运营总览"</div>
                            <h1>"管理端"</h1>
                        </div>
                        <div class="filter-group">
                            <button class="btn btn-outline btn-sm">"导出审计日志"</button>
                            <button class="btn btn-primary btn-sm">"发布公告"</button>
                        </div>
                    </div>
                    <div class="stat-grid">
                        {stats.into_iter().map(|stat| view! { <StatCard stat/> }).collect_view()}
                    </div>
                    <div class="admin-panels">
                        <section class="panel-card">
                            <h2>"用户管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按用户名或昵称搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"正常"</option><option>"已禁用"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"用户名"</th><th>"昵称"</th><th>"角色"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .users
                                            .into_iter()
                                            .map(|user| view! {
                                                <UserRow
                                                    user
                                                    session_id=user_table_session_id.clone()
                                                    disable_user_action
                                                    enable_user_action
                                                    disable_user_pending
                                                    enable_user_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = enable_user_result.get() {
                                        return match result {
                                            Ok(dashboard) => view! {
                                                <p class="success"><strong>"用户状态更新成功"</strong>{format!("当前 {} 个用户", dashboard.users.len())}</p>
                                            }.into_any(),
                                            Err(error) => view! {
                                                <p class="error"><strong>"用户状态更新失败"</strong>{error.to_string()}</p>
                                            }.into_any(),
                                        };
                                    }
                                    if let Some(result) = disable_user_result.get() {
                                        return match result {
                                            Ok(dashboard) => view! {
                                                <p class="success"><strong>"用户状态更新成功"</strong>{format!("当前 {} 个用户", dashboard.users.len())}</p>
                                            }.into_any(),
                                            Err(error) => view! {
                                                <p class="error"><strong>"用户状态更新失败"</strong>{error.to_string()}</p>
                                            }.into_any(),
                                        };
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"角色管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按角色编码或名称搜索"/>
                                <button class="btn btn-primary btn-sm">"创建角色"</button>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"角色编码"</th><th>"角色名称"</th><th>"权限"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        <tr><td>"admin"</td><td>"管理员"</td><td>"全部权限"</td><td><button class="btn btn-ghost btn-xs">"查看权限"</button></td></tr>
                                        <tr><td>"moderator"</td><td>"内容审核员"</td><td>"帖子 / 评论 / 举报"</td><td><button class="btn btn-ghost btn-xs">"更新角色"</button></td></tr>
                                        <tr><td>"operator"</td><td>"运营人员"</td><td>"公告 / 分类 / 标签"</td><td><button class="btn btn-ghost btn-xs">"删除角色"</button></td></tr>
                                    </tbody>
                                </table>
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"权限管理"</h2>
                            <div class="system-grid">
                                <a href="/admin">"user:view"<small>"查看用户"</small></a>
                                <a href="/admin">"role:create"<small>"创建角色"</small></a>
                                <a href="/admin">"role:update"<small>"更新角色"</small></a>
                                <a href="/admin">"role:delete"<small>"删除角色"</small></a>
                                <a href="/admin">"permission:view"<small>"查看权限"</small></a>
                                <a href="/admin">"audit:view"<small>"审计日志"</small></a>
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"帖子管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按标题或作者搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"草稿"</option><option>"已发布"</option><option>"已下架"</option><option>"已删除"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"标题"</th><th>"作者"</th><th>"分类"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .moderation_posts
                                            .into_iter()
                                            .map(|post| view! {
                                                <PostRow
                                                    post
                                                    session_id=post_table_session_id.clone()
                                                    take_down_post_action
                                                    restore_post_action
                                                    delete_post_action
                                                    pin_post_action
                                                    unpin_post_action
                                                    take_down_post_pending
                                                    restore_post_pending
                                                    delete_post_pending
                                                    pin_post_pending
                                                    unpin_post_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = unpin_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    if let Some(result) = pin_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    if let Some(result) = delete_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    if let Some(result) = restore_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    if let Some(result) = take_down_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"评论管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按评论内容或作者搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"正常"</option><option>"已删除"</option><option>"被举报"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"所属帖子"</th><th>"作者"</th><th>"内容"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .moderation_comments
                                            .into_iter()
                                            .map(|comment| view! {
                                                <CommentRow
                                                    comment
                                                    session_id=comment_table_session_id.clone()
                                                    delete_comment_action
                                                    recover_comment_action
                                                    delete_comment_pending
                                                    recover_comment_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = recover_comment_result.get() {
                                        return admin_comment_action_feedback(result);
                                    }
                                    if let Some(result) = delete_comment_result.get() {
                                        return admin_comment_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"治理队列"</h2>
                            <ul class="governance-list">
                                {queues.into_iter().map(|item| view! {
                                    <li><span>{item.label}</span><strong>{item.value}</strong></li>
                                }).collect_view()}
                            </ul>
                        </section>
                        <section class="panel-card">
                            <h2>"分类管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按分类名称搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"启用"</option><option>"停用"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"名称"</th><th>"颜色"</th><th>"排序"</th><th>"帖子数"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {categories.into_iter().map(|category| view! { <CategoryRow category/> }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"标签管理"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按标签名称搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"启用"</option><option>"禁用"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"名称"</th><th>"排序"</th><th>"使用数"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {tags.into_iter().map(|tag| view! { <TagRow tag/> }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"公告推送"</h2>
                            <div class="table-tools">
                                <input class="input input-bordered" placeholder="按公告标题搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"草稿"</option><option>"已发布"</option><option>"已下线"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"标题"</th><th>"类型"</th><th>"范围"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .announcements
                                            .into_iter()
                                            .map(|announcement| view! {
                                                <AnnouncementRow
                                                    announcement
                                                    session_id=announcement_table_session_id.clone()
                                                    publish_announcement_action
                                                    withdraw_announcement_action
                                                    publish_announcement_pending
                                                    withdraw_announcement_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = withdraw_announcement_result.get() {
                                        return admin_announcement_action_feedback(result);
                                    }
                                    if let Some(result) = publish_announcement_result.get() {
                                        return admin_announcement_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"举报处理"</h2>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"对象"</th><th>"类型"</th><th>"原因"</th><th>"举报人"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .reports
                                            .into_iter()
                                            .map(|report| view! {
                                                <ReportRow
                                                    report
                                                    session_id=report_table_session_id.clone()
                                                    handle_report_action
                                                    reject_report_action
                                                    handle_report_pending
                                                    reject_report_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = reject_report_result.get() {
                                        return admin_report_action_feedback(result);
                                    }
                                    if let Some(result) = handle_report_result.get() {
                                        return admin_report_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"审计日志"</h2>
                            <ul class="governance-list">
                                {audit_entries.into_iter().map(|entry| view! {
                                    <li>
                                        <span>{entry.actor} " · " {entry.action} " · " {entry.target} " · " {entry.ip} " · " {entry.user_agent}</span>
                                        <strong>{entry.time_label}</strong>
                                    </li>
                                }).collect_view()}
                            </ul>
                        </section>
                    </div>
            </section>
        </div>
    }
}

#[component]
fn UserRow(
    user: AdminUserRow,
    session_id: String,
    disable_user_action: ServerAction<DisableAdminUser>,
    enable_user_action: ServerAction<EnableAdminUser>,
    disable_user_pending: Memo<bool>,
    enable_user_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if user.status == "正常" {
        "badge badge-green"
    } else {
        "badge badge-gray"
    };
    let actions = user
        .actions
        .clone()
        .into_iter()
        .filter(|action| action != "禁用用户" && action != "解禁用户")
        .collect::<Vec<_>>();
    let status = user.status.clone();
    let is_disabled = status == "已禁用";
    let target_user_id = user.user_id.to_string();

    view! {
        <tr>
            <td>{user.username}</td>
            <td>{user.nickname}</td>
            <td>{user.roles.join(", ")}</td>
            <td><span class=badge_class>{user.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
                {if is_disabled {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=enable_user_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="target_user_id" value=user.user_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || enable_user_pending.get() || disabled_session_id.is_empty()
                            >
                                "解禁用户"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=disable_user_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="target_user_id" value=target_user_id/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || disable_user_pending.get() || disabled_session_id.is_empty()
                            >
                                "禁用用户"
                            </button>
                        </ActionForm>
                    }.into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn CommentRow(
    comment: AdminCommentRow,
    session_id: String,
    delete_comment_action: ServerAction<DeleteAdminComment>,
    recover_comment_action: ServerAction<RecoverAdminComment>,
    delete_comment_pending: Memo<bool>,
    recover_comment_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if comment.status == "正常" {
        "badge badge-green"
    } else {
        "badge badge-gray"
    };
    let actions = comment.actions.clone();
    let show_delete = actions.iter().any(|action| action == "删除评论");
    let show_recover = actions.iter().any(|action| action == "恢复评论");
    let show_view_post = actions.iter().any(|action| action == "查看帖子");

    view! {
        <tr>
            <td>{comment.post_title}</td>
            <td>{comment.author}</td>
            <td>{comment.content}</td>
            <td><span class=badge_class>{comment.status}</span></td>
            <td>
                {if show_delete {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=delete_comment_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="comment_id" value=comment.comment_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || delete_comment_pending.get() || disabled_session_id.is_empty()
                            >
                                "删除评论"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_recover {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=recover_comment_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="comment_id" value=comment.comment_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || recover_comment_pending.get() || disabled_session_id.is_empty()
                            >
                                "恢复评论"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_view_post {
                    view! { <button class="btn btn-ghost btn-xs">"查看帖子"</button> }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn CategoryRow(category: AdminCategoryRow) -> impl IntoView {
    let actions = category.actions.clone();

    view! {
        <tr>
            <td>{category.name}</td>
            <td><span class="category-swatch" style=format!("background: {}", category.color)></span>{category.color}</td>
            <td>{category.sort_order}</td>
            <td>{category.post_count}</td>
            <td><span class="badge badge-green">{category.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
            </td>
        </tr>
    }
}

#[component]
fn TagRow(tag: AdminTagRow) -> impl IntoView {
    let actions = tag.actions.clone();

    view! {
        <tr>
            <td>{tag.name}</td>
            <td>{tag.sort_order}</td>
            <td>{tag.use_count}</td>
            <td><span class="badge badge-blue">{tag.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
            </td>
        </tr>
    }
}

#[component]
fn AnnouncementRow(
    announcement: AdminAnnouncementRow,
    session_id: String,
    publish_announcement_action: ServerAction<PublishAdminAnnouncement>,
    withdraw_announcement_action: ServerAction<WithdrawAdminAnnouncement>,
    publish_announcement_pending: Memo<bool>,
    withdraw_announcement_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if announcement.status == "已发布" {
        "badge badge-green"
    } else {
        "badge badge-orange"
    };
    let actions = announcement.actions.clone();
    let show_publish = actions
        .iter()
        .any(|action| action == "发布公告" || action == "重新发布");
    let show_withdraw = actions.iter().any(|action| action == "下线公告");
    let show_push = actions.iter().any(|action| action == "推送公告");
    let show_edit = actions.iter().any(|action| action == "编辑");
    let publish_label = if announcement.status == "已下线" {
        "重新发布"
    } else {
        "发布公告"
    };

    view! {
        <tr>
            <td>{announcement.title}</td>
            <td>{announcement.announcement_type}</td>
            <td>{announcement.audience}</td>
            <td><span class=badge_class>{announcement.status}</span></td>
            <td>
                {if show_publish {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=publish_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=announcement.announcement_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || publish_announcement_pending.get() || disabled_session_id.is_empty()
                            >
                                {publish_label}
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_withdraw {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=withdraw_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=announcement.announcement_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || withdraw_announcement_pending.get() || disabled_session_id.is_empty()
                            >
                                "下线公告"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_push {
                    view! { <button class="btn btn-ghost btn-xs">"推送公告"</button> }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_edit {
                    view! { <button class="btn btn-ghost btn-xs">"编辑"</button> }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn ReportRow(
    report: AdminReportRow,
    session_id: String,
    handle_report_action: ServerAction<HandleAdminReport>,
    reject_report_action: ServerAction<RejectAdminReport>,
    handle_report_pending: Memo<bool>,
    reject_report_pending: Memo<bool>,
) -> impl IntoView {
    let actions = report.actions.clone();
    let show_handle = actions.iter().any(|action| action == "标记已处理");
    let show_reject = actions.iter().any(|action| action == "驳回");
    let show_detail = actions.iter().any(|action| action == "查看详情");

    view! {
        <tr>
            <td>{report.target}</td>
            <td>{report.target_type}</td>
            <td>{report.reason}</td>
            <td>{report.reporter}</td>
            <td><span class="badge badge-orange">{report.status}</span></td>
            <td>
                {if show_handle {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=handle_report_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="report_id" value=report.report_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || handle_report_pending.get() || disabled_session_id.is_empty()
                            >
                                "标记已处理"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_reject {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=reject_report_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="report_id" value=report.report_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || reject_report_pending.get() || disabled_session_id.is_empty()
                            >
                                "驳回"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_detail {
                    view! { <button class="btn btn-ghost btn-xs">"查看详情"</button> }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn StatCard(stat: AdminStat) -> impl IntoView {
    view! {
        <section class="stat-card">
            <span>{stat.label}</span>
            <strong>{stat.value}</strong>
            <small>{stat.delta}</small>
        </section>
    }
}

#[component]
fn PostRow(
    post: AdminPostRow,
    session_id: String,
    take_down_post_action: ServerAction<TakeDownAdminPost>,
    restore_post_action: ServerAction<RestoreAdminPost>,
    delete_post_action: ServerAction<DeleteAdminPost>,
    pin_post_action: ServerAction<PinAdminPost>,
    unpin_post_action: ServerAction<UnpinAdminPost>,
    take_down_post_pending: Memo<bool>,
    restore_post_pending: Memo<bool>,
    delete_post_pending: Memo<bool>,
    pin_post_pending: Memo<bool>,
    unpin_post_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if post.status == "已发布" {
        "badge badge-green"
    } else {
        "badge badge-orange"
    };
    let actions = post.actions.clone();
    let show_take_down = actions.iter().any(|action| action == "下架");
    let show_restore = actions.iter().any(|action| action == "恢复");
    let show_delete = actions.iter().any(|action| action == "删除");
    let show_pin = actions.iter().any(|action| action == "置顶");
    let show_unpin = actions.iter().any(|action| action == "取消置顶");
    let show_view = actions.iter().any(|action| action == "查看");

    view! {
        <tr>
            <td>{post.title}</td>
            <td>{post.author}</td>
            <td>{post.category}</td>
            <td><span class=badge_class>{post.status}</span></td>
            <td>
                {if show_take_down {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=take_down_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || take_down_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "下架"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_restore {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=restore_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || restore_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "恢复"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_pin {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=pin_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || pin_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "置顶"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_unpin {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=unpin_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || unpin_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "取消置顶"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_delete {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=delete_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || delete_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "删除"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_view {
                    view! { <button class="btn btn-ghost btn-xs">"查看"</button> }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

fn admin_post_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"帖子状态更新成功"</strong>{format!("当前 {} 篇帖子", dashboard.moderation_posts.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"帖子状态更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_comment_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"评论状态更新成功"</strong>{format!("当前 {} 条评论", dashboard.moderation_comments.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"评论状态更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_announcement_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"公告状态更新成功"</strong>{format!("当前 {} 条公告", dashboard.announcements.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"公告状态更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_report_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"举报处理成功"</strong>{format!("当前 {} 条举报", dashboard.reports.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"举报处理失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}
