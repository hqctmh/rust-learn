use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <PageShell>
            <div class="grid gap-6 lg:grid-cols-[240px_1fr]">
                <aside class="rounded-lg border border-base-300 p-4">
                    <ul class="menu">
                        <li><a class="active">"系统统计"</a></li>
                        <li><a>"帖子管理"</a></li>
                        <li><a>"评论管理"</a></li>
                        <li><a>"用户管理"</a></li>
                        <li><a>"角色权限"</a></li>
                        <li><a>"公告发布"</a></li>
                        <li><a>"举报处理"</a></li>
                        <li><a>"分类标签"</a></li>
                        <li><a>"文件管理"</a></li>
                        <li><a>"搜索索引"</a></li>
                        <li><a>"审计日志"</a></li>
                    </ul>
                </aside>
                <section class="space-y-6">
                    <div class="mb-4 flex items-center justify-between">
                        <h1 class="text-xl font-semibold">"管理端"</h1>
                        <div class="flex gap-2">
                            <button class="btn btn-outline btn-sm">"同步搜索索引"</button>
                            <button class="btn btn-primary btn-sm">"发布公告"</button>
                        </div>
                    </div>
                    <div class="grid gap-4 md:grid-cols-4">
                        <Stat label="用户总数" value="2"/>
                        <Stat label="帖子总数" value="3"/>
                        <Stat label="未处理举报" value="0"/>
                        <Stat label="审计日志" value="1"/>
                    </div>
                    <div class="rounded-lg border border-base-300 p-5">
                        <h2 class="mb-4 text-base font-semibold">"内容管理"</h2>
                        <div class="overflow-x-auto">
                            <table class="table table-zebra">
                                <thead><tr><th>"标题"</th><th>"作者"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                                <tbody>
                                    <tr><td>"Leptos + Axum 构建全栈应用"</td><td>"Skyline"</td><td><span class="badge badge-success">"已发布"</span></td><td><button class="btn btn-ghost btn-xs">"下架"</button></td></tr>
                                    <tr><td>"表单验证实践"</td><td>"hello-rust"</td><td><span class="badge badge-warning">"草稿"</span></td><td><button class="btn btn-ghost btn-xs">"查看"</button></td></tr>
                                </tbody>
                            </table>
                        </div>
                    </div>
                    <div class="grid gap-4 lg:grid-cols-2">
                        <Panel title="公告推送" body="发布公告后写入站内通知，后续由 NATS 和 WebSocket 完成实时 fanout。"/>
                        <Panel title="举报处理" body="举报可以标记已处理或驳回，处理动作会进入审计日志。"/>
                        <Panel title="文件上传" body="图片上传先完成 MIME、大小和 hash 元数据校验，RustFS 接入点已保留。"/>
                        <Panel title="搜索索引" body="帖子搜索 API 已可用，Elasticsearch 索引同步由后续消费者接入。"/>
                    </div>
                </section>
            </div>
        </PageShell>
    }
}

#[component]
fn Stat(label: &'static str, value: &'static str) -> impl IntoView {
    view! {
        <div class="stats rounded-lg border border-base-300 bg-base-100">
            <div class="stat">
                <div class="stat-title">{label}</div>
                <div class="stat-value text-primary">{value}</div>
            </div>
        </div>
    }
}

#[component]
fn Panel(title: &'static str, body: &'static str) -> impl IntoView {
    view! {
        <section class="rounded-lg border border-base-300 bg-base-100 p-5">
            <h2 class="mb-2 text-base font-semibold">{title}</h2>
            <p class="text-sm leading-6 text-base-content/70">{body}</p>
        </section>
    }
}
