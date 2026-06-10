use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn AdminPage() -> impl IntoView {
    view! {
        <PageShell>
            <div class="grid gap-6 lg:grid-cols-[240px_1fr]">
                <aside class="rounded-lg border border-base-300 p-4">
                    <ul class="menu">
                        <li><a class="active">"帖子管理"</a></li>
                        <li><a>"评论管理"</a></li>
                        <li><a>"用户管理"</a></li>
                        <li><a>"角色权限"</a></li>
                        <li><a>"公告发布"</a></li>
                        <li><a>"审计日志"</a></li>
                    </ul>
                </aside>
                <section class="rounded-lg border border-base-300 p-5">
                    <div class="mb-4 flex items-center justify-between">
                        <h1 class="text-xl font-semibold">"管理端"</h1>
                        <button class="btn btn-primary btn-sm">"发布公告"</button>
                    </div>
                    <div class="overflow-x-auto">
                        <table class="table table-zebra">
                            <thead><tr><th>"标题"</th><th>"作者"</th><th>"状态"</th><th>"操作"</th></tr></thead>
                            <tbody>
                                <tr><td>"Leptos + Axum 构建全栈应用"</td><td>"Skyline"</td><td><span class="badge badge-success">"已发布"</span></td><td><button class="btn btn-ghost btn-xs">"下架"</button></td></tr>
                                <tr><td>"表单验证实践"</td><td>"hello-rust"</td><td><span class="badge badge-warning">"草稿"</span></td><td><button class="btn btn-ghost btn-xs">"查看"</button></td></tr>
                            </tbody>
                        </table>
                    </div>
                </section>
            </div>
        </PageShell>
    }
}
