use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn PostDetailPage() -> impl IntoView {
    view! {
        <PageShell>
            <div class="detail-layout">
                <article class="detail-article">
                    <div class="breadcrumb">"首页 / 经验分享 / Leptos"</div>
                    <h1>"Leptos + Axum 构建全栈论坛的项目结构"</h1>
                    <div class="detail-meta">
                        <span class="avatar-mini">"S"</span>
                        <span>"Skyline"</span>
                        <span>"发布于 2026-06-10"</span>
                        <span>"更新于 2026-06-11"</span>
                        <span class="badge badge-blue">"经验分享"</span>
                        <span class="badge badge-soft">"leptos"</span>
                        <span class="badge badge-soft">"axum"</span>
                        <span class="badge badge-soft">"sqlx"</span>
                    </div>
                    <div class="article-stats">
                        <span>"浏览 1.1k"</span>
                        <span>"点赞 78"</span>
                        <span>"收藏 29"</span>
                        <span>"评论 15"</span>
                    </div>
                    <div class="prose article-body">
                        <p>"这篇帖子展示论坛系统的页面结构、认证边界、Markdown 渲染、评论树、通知事件和管理端权限模型。"</p>
                        <blockquote>"核心原则：页面可以隐藏按钮，但后端接口仍必须执行 RBAC 权限校验。"</blockquote>
                        <pre><code>"cargo leptos serve\ncargo test"</code></pre>
                        <p>"发布后，帖子内容进入 PostgreSQL，搜索索引通过 NATS 异步更新到 Elasticsearch，图片资源由 RustFS 存储。"</p>
                    </div>
                    <div class="detail-actions">
                        <button class="btn btn-primary">"点赞 78"</button>
                        <button class="btn btn-outline">"收藏 29"</button>
                        <button class="btn btn-outline">"关注作者"</button>
                        <button class="btn btn-ghost">"举报"</button>
                    </div>
                    <section class="comment-panel">
                        <div class="section-heading">
                            <h2>"评论与回复"</h2>
                            <a href="/login">"登录后互动"</a>
                        </div>
                        <textarea class="textarea textarea-bordered w-full" placeholder="写下你的评论，支持回复评论"></textarea>
                        <div class="comment-actions"><button class="btn btn-primary btn-sm">"发表评论"</button></div>
                        <CommentItem author="hello-rust" badge="作者回复" content="这个结构清晰，后续接 NATS 通知也比较自然。"/>
                        <CommentItem author="DreamMao" badge="2 条回复" content="搜索索引异步更新这块可以补一个失败重试队列。"/>
                    </section>
                </article>
                <aside class="side-stack">
                    <section class="panel-card">
                        <h2>"作者信息"</h2>
                        <div class="profile-card">
                            <span class="avatar-lg">"S"</span>
                            <strong>"Skyline"</strong>
                            <p>"Rust / Leptos 全栈开发者"</p>
                            <div class="profile-stats"><span>"关注 128"</span><span>"粉丝 2.4k"</span></div>
                        </div>
                    </section>
                    <section class="panel-card">
                        <h2>"相关推荐"</h2>
                        <ul class="related-list">
                            <li><a href="/">"server function 中使用 SQLx 事务"</a><small>"7 回复"</small></li>
                            <li><a href="/">"Markdown 渲染与代码高亮"</a><small>"3 回复"</small></li>
                            <li><a href="/">"Axum 中间件处理 request body"</a><small>"1 回复"</small></li>
                        </ul>
                    </section>
                    <section class="panel-card">
                        <h2>"权限提示"</h2>
                        <p class="muted-copy">"未登录用户可以查看内容和评论；点赞、收藏、关注、评论、回复和举报会引导登录。"</p>
                    </section>
                </aside>
            </div>
        </PageShell>
    }
}

#[component]
fn CommentItem(author: &'static str, badge: &'static str, content: &'static str) -> impl IntoView {
    view! {
        <div class="comment-item">
            <span class="avatar-mini">{author.chars().next().unwrap_or('P').to_string()}</span>
            <div>
                <div class="comment-meta"><strong>{author}</strong><span class="badge badge-soft">{badge}</span></div>
                <p>{content}</p>
                <div class="comment-toolrow">
                    <button class="btn btn-ghost btn-xs">"回复评论"</button>
                    <button class="btn btn-ghost btn-xs">"点赞评论"</button>
                    <button class="btn btn-ghost btn-xs">"删除自己的评论"</button>
                    <button class="btn btn-ghost btn-xs">"举报评论"</button>
                </div>
            </div>
        </div>
    }
}
