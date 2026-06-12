use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::admin::{
        AdminAnnouncementRow, AdminCategoryRow, AdminCommentRow, AdminDashboard, AdminPostRow,
        AdminReportRow, AdminStat, AdminTagRow, AdminUserRow,
    },
    page_data::{fallback_admin_dashboard, load_admin_dashboard},
};

#[component]
pub fn AdminPage() -> impl IntoView {
    let dashboard = Resource::new(|| (), |_| load_admin_dashboard());

    view! {
        <PageShell>
            <Suspense fallback=move || view! { <AdminDashboardView dashboard=fallback_admin_dashboard()/> }>
                {move || Suspend::new(async move {
                    let dashboard = dashboard.await.unwrap_or_else(|_| fallback_admin_dashboard());
                    view! { <AdminDashboardView dashboard/> }
                })}
            </Suspense>
        </PageShell>
    }
}

#[component]
fn AdminDashboardView(dashboard: AdminDashboard) -> impl IntoView {
    let menu = dashboard.menu.clone();
    let stats = dashboard.stats.clone();
    let users = dashboard.users.clone();
    let posts = dashboard.moderation_posts.clone();
    let comments = dashboard.moderation_comments.clone();
    let categories = dashboard.categories.clone();
    let tags = dashboard.tags.clone();
    let announcements = dashboard.announcements.clone();
    let reports = dashboard.reports.clone();
    let queues = dashboard.governance_queue.clone();
    let audit_entries = dashboard.audit_entries.clone();

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
                                        {users.into_iter().map(|user| view! { <UserRow user/> }).collect_view()}
                                    </tbody>
                                </table>
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
                                        {posts.into_iter().map(|post| view! { <PostRow post/> }).collect_view()}
                                    </tbody>
                                </table>
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
                                        {comments.into_iter().map(|comment| view! { <CommentRow comment/> }).collect_view()}
                                    </tbody>
                                </table>
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
                                        {announcements.into_iter().map(|announcement| view! { <AnnouncementRow announcement/> }).collect_view()}
                                    </tbody>
                                </table>
                            </div>
                        </section>
                        <section class="panel-card">
                            <h2>"举报处理"</h2>
                            <div class="overflow-x-auto">
                                <table class="table">
                                    <thead><tr><th>"对象"</th><th>"类型"</th><th>"原因"</th><th>"举报人"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                    <tbody>
                                        {reports.into_iter().map(|report| view! { <ReportRow report/> }).collect_view()}
                                    </tbody>
                                </table>
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
fn UserRow(user: AdminUserRow) -> impl IntoView {
    let badge_class = if user.status == "正常" {
        "badge badge-green"
    } else {
        "badge badge-gray"
    };
    let actions = user.actions.clone();

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
            </td>
        </tr>
    }
}

#[component]
fn CommentRow(comment: AdminCommentRow) -> impl IntoView {
    let badge_class = if comment.status == "正常" {
        "badge badge-green"
    } else {
        "badge badge-gray"
    };
    let actions = comment.actions.clone();

    view! {
        <tr>
            <td>{comment.post_title}</td>
            <td>{comment.author}</td>
            <td>{comment.content}</td>
            <td><span class=badge_class>{comment.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
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
fn AnnouncementRow(announcement: AdminAnnouncementRow) -> impl IntoView {
    let badge_class = if announcement.status == "已发布" {
        "badge badge-green"
    } else {
        "badge badge-orange"
    };
    let actions = announcement.actions.clone();

    view! {
        <tr>
            <td>{announcement.title}</td>
            <td>{announcement.announcement_type}</td>
            <td>{announcement.audience}</td>
            <td><span class=badge_class>{announcement.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
            </td>
        </tr>
    }
}

#[component]
fn ReportRow(report: AdminReportRow) -> impl IntoView {
    let actions = report.actions.clone();

    view! {
        <tr>
            <td>{report.target}</td>
            <td>{report.target_type}</td>
            <td>{report.reason}</td>
            <td>{report.reporter}</td>
            <td><span class="badge badge-orange">{report.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
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
fn PostRow(post: AdminPostRow) -> impl IntoView {
    let badge_class = if post.status == "已发布" {
        "badge badge-green"
    } else {
        "badge badge-orange"
    };
    let actions = post.actions.clone();

    view! {
        <tr>
            <td>{post.title}</td>
            <td>{post.author}</td>
            <td>{post.category}</td>
            <td><span class=badge_class>{post.status}</span></td>
            <td>
                {actions.into_iter().map(|action| view! {
                    <button class="btn btn-ghost btn-xs">{action}</button>
                }).collect_view()}
            </td>
        </tr>
    }
}
