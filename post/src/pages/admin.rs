use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;

use crate::{
    components::PageShell,
    domain::admin::{
        AdminAnnouncementRow, AdminCategoryRow, AdminCommentRow, AdminDashboard, AdminPostRow,
        AdminReportRow, AdminStat, AdminTagRow, AdminUserRow, audit_entries_csv,
    },
    domain::rbac::Role,
    page_data::{
        CreateAdminAnnouncement, CreateAdminCategory, CreateAdminRole, CreateAdminTag,
        DeleteAdminComment, DeleteAdminPost, DeleteAdminRole, DisableAdminCategory,
        DisableAdminTag, DisableAdminUser, EnableAdminCategory, EnableAdminTag, EnableAdminUser,
        HandleAdminReport, LockAdminPost, MergeAdminTag, PinAdminPost, PublishAdminAnnouncement,
        PushAdminAnnouncement, RecoverAdminComment, RejectAdminReport, RestoreAdminPost,
        TakeDownAdminPost, UnlockAdminPost, UnpinAdminPost, UpdateAdminAnnouncement,
        UpdateAdminCategory, UpdateAdminRole, UpdateAdminTag, UpdateAdminUserRoles,
        WithdrawAdminAnnouncement, fallback_admin_dashboard, load_admin_dashboard,
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
    let update_user_roles_action = ServerAction::<UpdateAdminUserRoles>::new();
    let take_down_post_action = ServerAction::<TakeDownAdminPost>::new();
    let restore_post_action = ServerAction::<RestoreAdminPost>::new();
    let delete_post_action = ServerAction::<DeleteAdminPost>::new();
    let pin_post_action = ServerAction::<PinAdminPost>::new();
    let unpin_post_action = ServerAction::<UnpinAdminPost>::new();
    let lock_post_action = ServerAction::<LockAdminPost>::new();
    let unlock_post_action = ServerAction::<UnlockAdminPost>::new();
    let delete_comment_action = ServerAction::<DeleteAdminComment>::new();
    let recover_comment_action = ServerAction::<RecoverAdminComment>::new();
    let create_role_action = ServerAction::<CreateAdminRole>::new();
    let update_role_action = ServerAction::<UpdateAdminRole>::new();
    let delete_role_action = ServerAction::<DeleteAdminRole>::new();
    let handle_report_action = ServerAction::<HandleAdminReport>::new();
    let reject_report_action = ServerAction::<RejectAdminReport>::new();
    let create_announcement_action = ServerAction::<CreateAdminAnnouncement>::new();
    let update_announcement_action = ServerAction::<UpdateAdminAnnouncement>::new();
    let push_announcement_action = ServerAction::<PushAdminAnnouncement>::new();
    let publish_announcement_action = ServerAction::<PublishAdminAnnouncement>::new();
    let withdraw_announcement_action = ServerAction::<WithdrawAdminAnnouncement>::new();
    let create_category_action = ServerAction::<CreateAdminCategory>::new();
    let update_category_action = ServerAction::<UpdateAdminCategory>::new();
    let enable_category_action = ServerAction::<EnableAdminCategory>::new();
    let disable_category_action = ServerAction::<DisableAdminCategory>::new();
    let create_tag_action = ServerAction::<CreateAdminTag>::new();
    let update_tag_action = ServerAction::<UpdateAdminTag>::new();
    let enable_tag_action = ServerAction::<EnableAdminTag>::new();
    let disable_tag_action = ServerAction::<DisableAdminTag>::new();
    let merge_tag_action = ServerAction::<MergeAdminTag>::new();
    let disable_user_pending = disable_user_action.pending();
    let enable_user_pending = enable_user_action.pending();
    let update_user_roles_pending = update_user_roles_action.pending();
    let take_down_post_pending = take_down_post_action.pending();
    let restore_post_pending = restore_post_action.pending();
    let delete_post_pending = delete_post_action.pending();
    let pin_post_pending = pin_post_action.pending();
    let unpin_post_pending = unpin_post_action.pending();
    let lock_post_pending = lock_post_action.pending();
    let unlock_post_pending = unlock_post_action.pending();
    let delete_comment_pending = delete_comment_action.pending();
    let recover_comment_pending = recover_comment_action.pending();
    let create_role_pending = create_role_action.pending();
    let update_role_pending = update_role_action.pending();
    let delete_role_pending = delete_role_action.pending();
    let handle_report_pending = handle_report_action.pending();
    let reject_report_pending = reject_report_action.pending();
    let create_announcement_pending = create_announcement_action.pending();
    let update_announcement_pending = update_announcement_action.pending();
    let push_announcement_pending = push_announcement_action.pending();
    let publish_announcement_pending = publish_announcement_action.pending();
    let withdraw_announcement_pending = withdraw_announcement_action.pending();
    let create_category_pending = create_category_action.pending();
    let update_category_pending = update_category_action.pending();
    let enable_category_pending = enable_category_action.pending();
    let disable_category_pending = disable_category_action.pending();
    let create_tag_pending = create_tag_action.pending();
    let update_tag_pending = update_tag_action.pending();
    let enable_tag_pending = enable_tag_action.pending();
    let disable_tag_pending = disable_tag_action.pending();
    let merge_tag_pending = merge_tag_action.pending();
    let disable_user_result = disable_user_action.value();
    let enable_user_result = enable_user_action.value();
    let update_user_roles_result = update_user_roles_action.value();
    let take_down_post_result = take_down_post_action.value();
    let restore_post_result = restore_post_action.value();
    let delete_post_result = delete_post_action.value();
    let pin_post_result = pin_post_action.value();
    let unpin_post_result = unpin_post_action.value();
    let lock_post_result = lock_post_action.value();
    let unlock_post_result = unlock_post_action.value();
    let delete_comment_result = delete_comment_action.value();
    let recover_comment_result = recover_comment_action.value();
    let create_role_result = create_role_action.value();
    let update_role_result = update_role_action.value();
    let delete_role_result = delete_role_action.value();
    let handle_report_result = handle_report_action.value();
    let reject_report_result = reject_report_action.value();
    let create_announcement_result = create_announcement_action.value();
    let update_announcement_result = update_announcement_action.value();
    let push_announcement_result = push_announcement_action.value();
    let publish_announcement_result = publish_announcement_action.value();
    let withdraw_announcement_result = withdraw_announcement_action.value();
    let create_category_result = create_category_action.value();
    let update_category_result = update_category_action.value();
    let enable_category_result = enable_category_action.value();
    let disable_category_result = disable_category_action.value();
    let create_tag_result = create_tag_action.value();
    let update_tag_result = update_tag_action.value();
    let enable_tag_result = enable_tag_action.value();
    let disable_tag_result = disable_tag_action.value();
    let merge_tag_result = merge_tag_action.value();
    let initial_dashboard = dashboard.clone();
    let current_dashboard = Memo::new(move |_| {
        if let Some(Ok(dashboard)) = merge_tag_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = create_tag_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = update_tag_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = create_category_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = update_category_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = disable_tag_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = enable_tag_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = disable_category_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = enable_category_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = delete_role_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = update_role_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = create_role_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = create_announcement_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = update_announcement_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = push_announcement_result.get() {
            return dashboard;
        }
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
        if let Some(Ok(dashboard)) = unlock_post_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = lock_post_result.get() {
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
        if let Some(Ok(dashboard)) = update_user_roles_result.get() {
            return dashboard;
        }
        if let Some(Ok(dashboard)) = disable_user_result.get() {
            return dashboard;
        }
        initial_dashboard.clone()
    });
    let menu = dashboard.menu.clone();
    let stats = dashboard.stats.clone();
    let queues = dashboard.governance_queue.clone();
    let audit_entries = dashboard.audit_entries.clone();
    let audit_csv_href = format!(
        "data:text/csv;charset=utf-8,{}",
        percent_encode_data_uri(&audit_entries_csv(&audit_entries))
    );
    let user_table_session_id = session_id.clone();
    let role_table_session_id = session_id.clone();
    let role_create_session_id = session_id.clone();
    let role_create_disabled_session_id = session_id.clone();
    let post_table_session_id = session_id.clone();
    let comment_table_session_id = session_id.clone();
    let category_table_session_id = session_id.clone();
    let category_create_session_id = session_id.clone();
    let category_create_disabled_session_id = session_id.clone();
    let tag_table_session_id = session_id.clone();
    let tag_create_session_id = session_id.clone();
    let tag_create_disabled_session_id = session_id.clone();
    let announcement_table_session_id = session_id.clone();
    let announcement_create_session_id = session_id.clone();
    let announcement_create_disabled_session_id = session_id.clone();
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
                            <a
                                class="btn btn-outline btn-sm"
                                href=audit_csv_href
                                download="audit-logs.csv"
                            >
                                "导出审计日志"
                            </a>
                            <a class="btn btn-primary btn-sm" href="#admin-announcements">"发布公告"</a>
                        </div>
                    </div>
                    <div class="stat-grid">
                        {stats.into_iter().map(|stat| view! { <StatCard stat/> }).collect_view()}
                    </div>
                    <div class="admin-panels">
                        <section class="panel-card" id="admin-users">
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
                                                    update_user_roles_action
                                                    disable_user_pending
                                                    enable_user_pending
                                                    update_user_roles_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = update_user_roles_result.get() {
                                        return match result {
                                            Ok(dashboard) => view! {
                                                <p class="success"><strong>"用户角色更新成功"</strong>{format!("当前 {} 个用户", dashboard.users.len())}</p>
                                            }.into_any(),
                                            Err(error) => view! {
                                                <p class="error"><strong>"用户角色更新失败"</strong>{error.to_string()}</p>
                                            }.into_any(),
                                        };
                                    }
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
                        <section class="panel-card" id="admin-roles">
                            <h2>"角色管理"</h2>
                            <div class="table-tools">
                                <ActionForm action=create_role_action>
                                    <input type="hidden" name="session_id" value=role_create_session_id/>
                                    <input class="input input-bordered" name="code" placeholder="角色编码"/>
                                    <input class="input input-bordered" name="name" placeholder="角色名称"/>
                                    <input class="input input-bordered" name="permission_codes" placeholder="权限编码，逗号分隔"/>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        type="submit"
                                        disabled=move || create_role_pending.get() || role_create_disabled_session_id.is_empty()
                                    >
                                        "创建角色"
                                    </button>
                                </ActionForm>
                                <input class="input input-bordered" placeholder="按角色编码或名称搜索"/>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"角色编码"</th><th>"角色名称"</th><th>"权限"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || {
                                            let roles = current_dashboard.get().roles;
                                            roles
                                                .into_iter()
                                                .map(|role| view! {
                                                    <RoleRow
                                                        role
                                                        session_id=role_table_session_id.clone()
                                                        update_role_action
                                                        delete_role_action
                                                        update_role_pending
                                                        delete_role_pending
                                                    />
                                                })
                                                .collect_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = create_role_result.get() {
                                        return admin_role_create_feedback(result);
                                    }
                                    if let Some(result) = update_role_result.get() {
                                        return admin_role_update_feedback(result);
                                    }
                                    if let Some(result) = delete_role_result.get() {
                                        return admin_role_delete_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card" id="admin-permissions">
                            <h2>"权限管理"</h2>
                            <div class="system-grid">
                                <a href="#admin-users">"user:view"<small>"查看用户"</small></a>
                                <a href="#admin-roles">"role:create"<small>"创建角色"</small></a>
                                <a href="#admin-roles">"role:update"<small>"更新角色"</small></a>
                                <a href="#admin-roles">"role:delete"<small>"删除角色"</small></a>
                                <a href="#admin-posts">"post:update"<small>"帖子管理"</small></a>
                                <a href="#admin-permissions">"permission:view"<small>"查看权限"</small></a>
                                <a href="#admin-audit">"audit:view"<small>"审计日志"</small></a>
                            </div>
                        </section>
                        <section class="panel-card" id="admin-posts">
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
                                                    lock_post_action
                                                    unlock_post_action
                                                    take_down_post_pending
                                                    restore_post_pending
                                                    delete_post_pending
                                                    pin_post_pending
                                                    unpin_post_pending
                                                    lock_post_pending
                                                    unlock_post_pending
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
                                    if let Some(result) = unlock_post_result.get() {
                                        return admin_post_action_feedback(result);
                                    }
                                    if let Some(result) = lock_post_result.get() {
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
                        <section class="panel-card" id="admin-comments">
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
                                <ActionForm action=create_category_action>
                                    <input type="hidden" name="session_id" value=category_create_session_id/>
                                    <input class="input input-bordered" name="name" placeholder="分类名称"/>
                                    <input class="input input-bordered" name="color" placeholder="#0064E0"/>
                                    <input class="input input-bordered" name="sort_order" type="number" value=10/>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        type="submit"
                                        disabled=move || create_category_pending.get() || category_create_disabled_session_id.is_empty()
                                    >
                                        "创建分类"
                                    </button>
                                </ActionForm>
                                <input class="input input-bordered" placeholder="按分类名称搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"启用"</option><option>"停用"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"名称"</th><th>"颜色"</th><th>"排序"</th><th>"帖子数"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || current_dashboard
                                            .get()
                                            .categories
                                            .into_iter()
                                            .map(|category| view! {
                                                <CategoryRow
                                                    category
                                                    session_id=category_table_session_id.clone()
                                                    update_category_action
                                                    enable_category_action
                                                    disable_category_action
                                                    update_category_pending
                                                    enable_category_pending
                                                    disable_category_pending
                                                />
                                            })
                                            .collect_view()}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = create_category_result.get() {
                                        return admin_category_create_feedback(result);
                                    }
                                    if let Some(result) = update_category_result.get() {
                                        return admin_category_update_feedback(result);
                                    }
                                    if let Some(result) = disable_category_result.get() {
                                        return admin_category_action_feedback(result);
                                    }
                                    if let Some(result) = enable_category_result.get() {
                                        return admin_category_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"标签管理"</h2>
                            <div class="table-tools">
                                <ActionForm action=create_tag_action>
                                    <input type="hidden" name="session_id" value=tag_create_session_id/>
                                    <input class="input input-bordered" name="name" placeholder="标签名称"/>
                                    <input class="input input-bordered" name="sort_order" type="number" value=10/>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        type="submit"
                                        disabled=move || create_tag_pending.get() || tag_create_disabled_session_id.is_empty()
                                    >
                                        "创建标签"
                                    </button>
                                </ActionForm>
                                <input class="input input-bordered" placeholder="按标签名称搜索"/>
                                <select class="select select-bordered"><option>"全部状态"</option><option>"启用"</option><option>"禁用"</option></select>
                            </div>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"名称"</th><th>"排序"</th><th>"使用数"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {move || {
                                            let tag_options = current_dashboard.get().tags;
                                            tag_options
                                                .clone()
                                                .into_iter()
                                                .map(|tag| view! {
                                                <TagRow
                                                    tag
                                                    tag_options=tag_options.clone()
                                                    session_id=tag_table_session_id.clone()
                                                    update_tag_action
                                                    enable_tag_action
                                                    disable_tag_action
                                                    merge_tag_action
                                                    update_tag_pending
                                                    enable_tag_pending
                                                    disable_tag_pending
                                                    merge_tag_pending
                                                />
                                            })
                                            .collect_view()
                                        }}
                                    </tbody>
                                </table>
                            </div>
                            <div class="profile-action-feedback">
                                {move || {
                                    if let Some(result) = create_tag_result.get() {
                                        return admin_tag_create_feedback(result);
                                    }
                                    if let Some(result) = merge_tag_result.get() {
                                        return admin_tag_merge_feedback(result);
                                    }
                                    if let Some(result) = update_tag_result.get() {
                                        return admin_tag_update_feedback(result);
                                    }
                                    if let Some(result) = disable_tag_result.get() {
                                        return admin_tag_action_feedback(result);
                                    }
                                    if let Some(result) = enable_tag_result.get() {
                                        return admin_tag_action_feedback(result);
                                    }
                                    ().into_any()
                                }}
                            </div>
                        </section>
                        <section class="panel-card" id="admin-announcements">
                            <h2>"公告推送"</h2>
                            <div class="table-tools">
                                <ActionForm action=create_announcement_action>
                                    <input type="hidden" name="session_id" value=announcement_create_session_id/>
                                    <input class="input input-bordered" name="title" placeholder="公告标题"/>
                                    <input class="input input-bordered" name="content" placeholder="公告内容"/>
                                    <input class="input input-bordered" name="announcement_type" placeholder="公告类型"/>
                                    <input class="input input-bordered" name="effective_at" type="datetime-local" placeholder="生效时间"/>
                                    <input class="input input-bordered" name="expires_at" type="datetime-local" placeholder="失效时间"/>
                                    <select class="select select-bordered" name="pinned">
                                        <option value="false">"不置顶"</option>
                                        <option value="true">"置顶"</option>
                                    </select>
                                    <button
                                        class="btn btn-primary btn-sm"
                                        type="submit"
                                        disabled=move || create_announcement_pending.get() || announcement_create_disabled_session_id.is_empty()
                                    >
                                        "发布公告"
                                    </button>
                                </ActionForm>
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
                                                    update_announcement_action
                                                    push_announcement_action
                                                    publish_announcement_action
                                                    withdraw_announcement_action
                                                    update_announcement_pending
                                                    push_announcement_pending
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
                                    if let Some(result) = create_announcement_result.get() {
                                        return admin_announcement_create_feedback(result);
                                    }
                                    if let Some(result) = update_announcement_result.get() {
                                        return admin_announcement_update_feedback(result);
                                    }
                                    if let Some(result) = push_announcement_result.get() {
                                        return admin_announcement_push_feedback(result);
                                    }
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
                        <section class="panel-card" id="admin-audit">
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
    update_user_roles_action: ServerAction<UpdateAdminUserRoles>,
    disable_user_pending: Memo<bool>,
    enable_user_pending: Memo<bool>,
    update_user_roles_pending: Memo<bool>,
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
        .filter(|action| action != "禁用用户" && action != "解禁用户" && action != "调整角色")
        .collect::<Vec<_>>();
    let show_role_update = user.actions.iter().any(|action| action == "调整角色");
    let status = user.status.clone();
    let is_disabled = status == "已禁用";
    let target_user_id = user.user_id.to_string();
    let role_value = user.roles.join(",");

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
                {if show_role_update {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=update_user_roles_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="target_user_id" value=user.user_id.to_string()/>
                            <label class="admin-inline-action">
                                <span>"角色"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="roles"
                                    placeholder="角色编码，逗号分隔"
                                    value=role_value
                                />
                            </label>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || update_user_roles_pending.get() || disabled_session_id.is_empty()
                            >
                                "调整角色"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
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
fn RoleRow(
    role: Role,
    session_id: String,
    update_role_action: ServerAction<UpdateAdminRole>,
    delete_role_action: ServerAction<DeleteAdminRole>,
    update_role_pending: Memo<bool>,
    delete_role_pending: Memo<bool>,
) -> impl IntoView {
    let permission_count = role.permissions.len();
    let permission_summary = role_permission_summary(&role);
    let display_role_code = role.code.clone();
    let update_role_code = role.code.clone();
    let delete_role_code = role.code.clone();
    let role_name = role.name.clone();
    let permission_codes = role_permission_codes_value(&role);
    let permission_details = role.permissions.iter().cloned().collect::<Vec<_>>();
    let update_session_id = session_id.clone();
    let update_disabled_session_id = session_id.clone();
    let delete_session_id = session_id.clone();
    let delete_disabled_session_id = session_id;

    view! {
        <tr>
            <td>{display_role_code}</td>
            <td>{role_name.clone()}</td>
            <td>{permission_summary}<small>{format!("{} 项权限", permission_count)}</small></td>
            <td>
                <details class="admin-inline-details">
                    <summary class="btn btn-ghost btn-xs">"查看权限"</summary>
                    <ul class="permission-list">
                        {permission_details
                            .into_iter()
                            .map(|permission| {
                                let code = permission.code.clone();
                                let name = permission.name.clone();
                                view! { <li><code>{code}</code><span>{name}</span></li> }
                            })
                            .collect_view()}
                    </ul>
                </details>
                <ActionForm action=update_role_action>
                    <input type="hidden" name="session_id" value=update_session_id/>
                    <input type="hidden" name="role_code" value=update_role_code/>
                    <label class="admin-inline-action">
                        <span>"名称"</span>
                        <input class="input input-bordered input-xs" name="name" value=role_name.clone()/>
                    </label>
                    <label class="admin-inline-action">
                        <span>"权限"</span>
                        <input
                            class="input input-bordered input-xs"
                            name="permission_codes"
                            value=permission_codes.clone()
                        />
                    </label>
                    <button
                        class="btn btn-ghost btn-xs"
                        type="submit"
                        disabled=move || update_role_pending.get() || update_disabled_session_id.is_empty()
                    >
                        "更新角色"
                    </button>
                </ActionForm>
                <ActionForm action=delete_role_action>
                    <input type="hidden" name="session_id" value=delete_session_id/>
                    <input type="hidden" name="role_code" value=delete_role_code/>
                    <button
                        class="btn btn-ghost btn-xs"
                        type="submit"
                        disabled=move || delete_role_pending.get() || delete_disabled_session_id.is_empty()
                    >
                        "删除角色"
                    </button>
                </ActionForm>
            </td>
        </tr>
    }
}

fn role_permission_codes_value(role: &Role) -> String {
    role.permissions
        .iter()
        .map(|permission| permission.code.as_str())
        .collect::<Vec<_>>()
        .join(",")
}

fn percent_encode_data_uri(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn datetime_local_value(value: Option<time::OffsetDateTime>) -> String {
    value
        .map(|value| {
            format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}",
                value.year(),
                value.month() as u8,
                value.day(),
                value.hour(),
                value.minute()
            )
        })
        .unwrap_or_default()
}

fn role_permission_summary(role: &Role) -> String {
    if role.permissions.len() >= 10 {
        return "全部权限".to_string();
    }

    let summary = role
        .permissions
        .iter()
        .take(3)
        .map(|permission| permission.name.as_str())
        .collect::<Vec<_>>()
        .join(" / ");
    if role.permissions.len() > 3 {
        format!("{summary} / +{}", role.permissions.len() - 3)
    } else {
        summary
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
    let comment_post_href = format!("/posts/{}", comment.post_id);

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
                    view! { <a class="btn btn-ghost btn-xs" href=comment_post_href>"查看帖子"</a> }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn CategoryRow(
    category: AdminCategoryRow,
    session_id: String,
    update_category_action: ServerAction<UpdateAdminCategory>,
    enable_category_action: ServerAction<EnableAdminCategory>,
    disable_category_action: ServerAction<DisableAdminCategory>,
    update_category_pending: Memo<bool>,
    enable_category_pending: Memo<bool>,
    disable_category_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if category.status == "启用" {
        "badge badge-green"
    } else {
        "badge badge-gray"
    };
    let actions = category.actions.clone();
    let show_update = actions
        .iter()
        .any(|action| action == "编辑" || action == "调整排序");
    let show_enable = actions.iter().any(|action| action == "启用");
    let show_disable = actions.iter().any(|action| action == "停用");
    let passive_actions = actions
        .into_iter()
        .filter(|action| {
            action != "启用" && action != "停用" && action != "编辑" && action != "调整排序"
        })
        .collect::<Vec<_>>();

    view! {
        <tr>
            <td>{category.name.clone()}</td>
            <td><span class="category-swatch" style=format!("background: {}", category.color)></span>{category.color.clone()}</td>
            <td>{category.sort_order}</td>
            <td>{category.post_count}</td>
            <td><span class=badge_class>{category.status}</span></td>
            <td>
                {passive_actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
                {if show_update {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=update_category_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="category_id" value=category.category_id.to_string()/>
                            <label class="admin-inline-action">
                                <span>"名称"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="name"
                                    value=category.name.clone()
                                />
                            </label>
                            <label class="admin-inline-action">
                                <span>"颜色"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="color"
                                    value=category.color.clone()
                                />
                            </label>
                            <label class="admin-inline-action">
                                <span>"排序"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="sort_order"
                                    type="number"
                                    value=category.sort_order
                                />
                            </label>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || update_category_pending.get() || disabled_session_id.is_empty()
                            >
                                "更新分类"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_enable {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=enable_category_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="category_id" value=category.category_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || enable_category_pending.get() || disabled_session_id.is_empty()
                            >
                                "启用"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_disable {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=disable_category_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="category_id" value=category.category_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || disable_category_pending.get() || disabled_session_id.is_empty()
                            >
                                "停用"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn TagRow(
    tag: AdminTagRow,
    tag_options: Vec<AdminTagRow>,
    session_id: String,
    update_tag_action: ServerAction<UpdateAdminTag>,
    enable_tag_action: ServerAction<EnableAdminTag>,
    disable_tag_action: ServerAction<DisableAdminTag>,
    merge_tag_action: ServerAction<MergeAdminTag>,
    update_tag_pending: Memo<bool>,
    enable_tag_pending: Memo<bool>,
    disable_tag_pending: Memo<bool>,
    merge_tag_pending: Memo<bool>,
) -> impl IntoView {
    let badge_class = if tag.status == "启用" {
        "badge badge-blue"
    } else {
        "badge badge-gray"
    };
    let actions = tag.actions.clone();
    let show_update = actions.iter().any(|action| action == "编辑");
    let show_enable = actions.iter().any(|action| action == "启用");
    let show_disable = actions.iter().any(|action| action == "禁用");
    let show_merge = actions.iter().any(|action| action == "合并标签");
    let merge_targets = tag_options
        .into_iter()
        .filter(|target| target.tag_id != tag.tag_id)
        .collect::<Vec<_>>();
    let passive_actions = actions
        .into_iter()
        .filter(|action| {
            action != "启用" && action != "禁用" && action != "合并标签" && action != "编辑"
        })
        .collect::<Vec<_>>();

    view! {
        <tr>
            <td>{tag.name.clone()}</td>
            <td>{tag.sort_order}</td>
            <td>{tag.use_count}</td>
            <td><span class=badge_class>{tag.status}</span></td>
            <td>
                {passive_actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
                {if show_update {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=update_tag_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="tag_id" value=tag.tag_id.to_string()/>
                            <label class="admin-inline-action">
                                <span>"名称"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="name"
                                    value=tag.name.clone()
                                />
                            </label>
                            <label class="admin-inline-action">
                                <span>"排序"</span>
                                <input
                                    class="input input-bordered input-xs"
                                    name="sort_order"
                                    type="number"
                                    value=tag.sort_order
                                />
                            </label>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || update_tag_pending.get() || disabled_session_id.is_empty()
                            >
                                "更新标签"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_merge && !merge_targets.is_empty() {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=merge_tag_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="source_tag_id" value=tag.tag_id.to_string()/>
                            <label class="admin-inline-action">
                                <span>"合并到"</span>
                                <select class="select select-bordered select-xs" name="target_tag_id">
                                    {merge_targets.into_iter().map(|target| view! {
                                        <option value=target.tag_id.to_string()>{target.name}</option>
                                    }).collect_view()}
                                </select>
                            </label>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || merge_tag_pending.get() || disabled_session_id.is_empty()
                            >
                                "合并标签"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_enable {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=enable_tag_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="tag_id" value=tag.tag_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || enable_tag_pending.get() || disabled_session_id.is_empty()
                            >
                                "启用"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_disable {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=disable_tag_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="tag_id" value=tag.tag_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || disable_tag_pending.get() || disabled_session_id.is_empty()
                            >
                                "禁用"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
            </td>
        </tr>
    }
}

#[component]
fn AnnouncementRow(
    announcement: AdminAnnouncementRow,
    session_id: String,
    update_announcement_action: ServerAction<UpdateAdminAnnouncement>,
    push_announcement_action: ServerAction<PushAdminAnnouncement>,
    publish_announcement_action: ServerAction<PublishAdminAnnouncement>,
    withdraw_announcement_action: ServerAction<WithdrawAdminAnnouncement>,
    update_announcement_pending: Memo<bool>,
    push_announcement_pending: Memo<bool>,
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
    let announcement_id = announcement.announcement_id.to_string();
    let title = announcement.title.clone();
    let content = announcement.content.clone();
    let announcement_type = announcement.announcement_type.clone();
    let audience = announcement.audience.clone();
    let status = announcement.status.clone();
    let pinned = announcement.pinned;
    let effective_at = datetime_local_value(announcement.effective_at);
    let expires_at = datetime_local_value(announcement.expires_at);

    view! {
        <tr>
            <td>{title.clone()}</td>
            <td>{announcement_type.clone()}</td>
            <td>{audience}</td>
            <td><span class=badge_class>{status}</span></td>
            <td>
                {if show_publish {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    let form_announcement_id = announcement_id.clone();
                    view! {
                        <ActionForm action=publish_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=form_announcement_id/>
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
                    let form_announcement_id = announcement_id.clone();
                    view! {
                        <ActionForm action=withdraw_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=form_announcement_id/>
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
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    let form_announcement_id = announcement_id.clone();
                    view! {
                        <ActionForm action=push_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=form_announcement_id/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || push_announcement_pending.get() || disabled_session_id.is_empty()
                            >
                                "推送公告"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_edit {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    let form_announcement_id = announcement_id.clone();
                    let edit_title = title.clone();
                    let edit_content = content.clone();
                    let edit_type = announcement_type.clone();
                    let edit_effective_at = effective_at.clone();
                    let edit_expires_at = expires_at.clone();
                    view! {
                        <ActionForm action=update_announcement_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="announcement_id" value=form_announcement_id/>
                            <input class="input input-bordered input-xs" name="title" value=edit_title/>
                            <input class="input input-bordered input-xs" name="content" value=edit_content/>
                            <input class="input input-bordered input-xs" name="announcement_type" value=edit_type/>
                            <input class="input input-bordered input-xs" name="effective_at" type="datetime-local" placeholder="生效时间" value=edit_effective_at/>
                            <input class="input input-bordered input-xs" name="expires_at" type="datetime-local" placeholder="失效时间" value=edit_expires_at/>
                            <select class="select select-bordered select-xs" name="pinned">
                                <option value="false" selected=!pinned>"不置顶"</option>
                                <option value="true" selected=pinned>"置顶"</option>
                            </select>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || update_announcement_pending.get() || disabled_session_id.is_empty()
                            >
                                "编辑公告"
                            </button>
                        </ActionForm>
                    }.into_any()
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
    let report_target = report.target.clone();
    let report_target_type = report.target_type.clone();
    let report_reason = report.reason.clone();
    let report_reporter = report.reporter.clone();
    let report_status = report.status.clone();

    view! {
        <tr>
            <td>{report_target.clone()}</td>
            <td>{report_target_type.clone()}</td>
            <td>{report_reason.clone()}</td>
            <td>{report_reporter.clone()}</td>
            <td><span class="badge badge-orange">{report_status.clone()}</span></td>
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
                    view! {
                        <details class="admin-inline-details">
                            <summary class="btn btn-ghost btn-xs">"查看详情"</summary>
                            <dl class="report-detail-list">
                                <dt>"对象"</dt><dd>{report_target}</dd>
                                <dt>"类型"</dt><dd>{report_target_type}</dd>
                                <dt>"原因"</dt><dd>{report_reason}</dd>
                                <dt>"举报人"</dt><dd>{report_reporter}</dd>
                                <dt>"状态"</dt><dd>{report_status}</dd>
                            </dl>
                        </details>
                    }.into_any()
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
    lock_post_action: ServerAction<LockAdminPost>,
    unlock_post_action: ServerAction<UnlockAdminPost>,
    take_down_post_pending: Memo<bool>,
    restore_post_pending: Memo<bool>,
    delete_post_pending: Memo<bool>,
    pin_post_pending: Memo<bool>,
    unpin_post_pending: Memo<bool>,
    lock_post_pending: Memo<bool>,
    unlock_post_pending: Memo<bool>,
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
    let show_lock = actions.iter().any(|action| action == "锁定");
    let show_unlock = actions.iter().any(|action| action == "解锁");
    let show_view = actions.iter().any(|action| action == "查看");
    let post_href = format!("/posts/{}", post.post_id);

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
                {if show_lock {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=lock_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || lock_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "锁定"
                            </button>
                        </ActionForm>
                    }.into_any()
                } else {
                    ().into_any()
                }}
                {if show_unlock {
                    let form_session_id = session_id.clone();
                    let disabled_session_id = session_id.clone();
                    view! {
                        <ActionForm action=unlock_post_action>
                            <input type="hidden" name="session_id" value=form_session_id/>
                            <input type="hidden" name="post_id" value=post.post_id.to_string()/>
                            <button
                                class="btn btn-ghost btn-xs"
                                type="submit"
                                disabled=move || unlock_post_pending.get() || disabled_session_id.is_empty()
                            >
                                "解锁"
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
                    view! { <a class="btn btn-ghost btn-xs" href=post_href>"查看"</a> }.into_any()
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

fn admin_role_create_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"角色创建成功"</strong>{format!("当前 {} 个角色", dashboard.roles.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"角色创建失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_role_update_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"角色更新成功"</strong>{format!("当前 {} 个角色", dashboard.roles.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"角色更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_role_delete_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"角色删除成功"</strong>{format!("当前 {} 个角色", dashboard.roles.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"角色删除失败"</strong>{error.to_string()}</p>
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

fn admin_announcement_create_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"公告创建成功"</strong>{format!("当前 {} 条公告", dashboard.announcements.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"公告创建失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_announcement_update_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"公告更新成功"</strong>{format!("当前 {} 条公告", dashboard.announcements.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"公告更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_announcement_push_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"公告推送成功"</strong>{format!("当前 {} 条公告", dashboard.announcements.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"公告推送失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_category_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"分类状态更新成功"</strong>{format!("当前 {} 个分类", dashboard.categories.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"分类状态更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_category_create_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"分类创建成功"</strong>{format!("当前 {} 个分类", dashboard.categories.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"分类创建失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_category_update_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"分类更新成功"</strong>{format!("当前 {} 个分类", dashboard.categories.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"分类更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_tag_action_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"标签状态更新成功"</strong>{format!("当前 {} 个标签", dashboard.tags.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"标签状态更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_tag_create_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"标签创建成功"</strong>{format!("当前 {} 个标签", dashboard.tags.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"标签创建失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_tag_update_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"标签更新成功"</strong>{format!("当前 {} 个标签", dashboard.tags.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"标签更新失败"</strong>{error.to_string()}</p>
        }
        .into_any(),
    }
}

fn admin_tag_merge_feedback(result: Result<AdminDashboard, ServerFnError>) -> AnyView {
    match result {
        Ok(dashboard) => view! {
            <p class="success"><strong>"标签合并成功"</strong>{format!("当前 {} 个标签", dashboard.tags.len())}</p>
        }
        .into_any(),
        Err(error) => view! {
            <p class="error"><strong>"标签合并失败"</strong>{error.to_string()}</p>
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
