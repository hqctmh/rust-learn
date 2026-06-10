use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn PostDetailPage() -> impl IntoView {
    view! {
        <PageShell>
            <article class="mx-auto max-w-4xl">
                <div class="mb-6 flex items-center justify-between gap-4">
                    <div>
                        <h1 class="text-3xl font-bold">"Leptos + Axum 构建全栈论坛"</h1>
                        <p class="mt-3 text-sm text-base-content/60">"Skyline · 实践 · 2026-06-10 · Leptos / Axum / SQLx"</p>
                    </div>
                    <button class="btn btn-outline btn-sm">"关注作者"</button>
                </div>
                <div class="prose max-w-none">
                    <p>"这篇帖子展示论坛系统的项目结构、认证边界、Markdown 渲染和评论模型。"</p>
                    <pre><code>"cargo leptos serve"</code></pre>
                </div>
                <div class="mt-8 flex gap-3">
                    <button class="btn btn-primary btn-sm">"点赞 78"</button>
                    <button class="btn btn-outline btn-sm">"收藏 29"</button>
                    <button class="btn btn-ghost btn-sm">"举报"</button>
                </div>
                <section class="mt-10 rounded-lg border border-base-300 p-5">
                    <h2 class="mb-4 text-lg font-semibold">"评论"</h2>
                    <textarea class="textarea textarea-bordered w-full" placeholder="写下你的评论"></textarea>
                    <div class="mt-3 text-right"><button class="btn btn-primary btn-sm">"发表评论"</button></div>
                    <div class="mt-6 border-t border-base-300 pt-5">
                        <p class="font-medium">"hello-rust"</p>
                        <p class="mt-2 text-sm text-base-content/70">"这个结构清晰，后续接 NATS 通知也比较自然。"</p>
                    </div>
                </section>
            </article>
        </PageShell>
    }
}
