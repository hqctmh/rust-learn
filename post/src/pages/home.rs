use leptos::prelude::*;

use crate::components::PageShell;
use crate::domain::posts::PostSummary;

#[component]
pub fn HomePage() -> impl IntoView {
    let posts = vec![
        PostSummary::sample(),
        PostSummary {
            title: "如何在 Leptos 中优雅地处理表单验证".to_string(),
            summary: "结合 server functions、validator 和服务端校验，整理一套发布帖子的输入边界。".to_string(),
            author_name: "hello-rust".to_string(),
            category_name: Some("教程".to_string()),
            view_count: 856,
            comment_count: 18,
            like_count: 96,
            favorite_count: 34,
            tags: vec!["leptos".to_string(), "form".to_string(), "validation".to_string()],
            ..PostSummary::sample()
        },
        PostSummary {
            title: "Leptos + Axum 构建全栈应用的项目结构分享".to_string(),
            summary: "一个可复用的项目结构，包含前端组件、后端 API、共享类型、认证和数据库模块。".to_string(),
            author_name: "Skyline".to_string(),
            category_name: Some("实践".to_string()),
            view_count: 642,
            comment_count: 15,
            like_count: 78,
            favorite_count: 29,
            tags: vec!["leptos".to_string(), "axum".to_string(), "fullstack".to_string()],
            ..PostSummary::sample()
        },
    ];

    view! {
        <PageShell>
            <div class="grid gap-8 lg:grid-cols-[1fr_360px]">
                <section>
                    <div class="tabs tabs-border mb-5">
                        <a class="tab tab-active text-primary">"推荐"</a>
                        <a class="tab">"最新"</a>
                        <a class="tab">"热门"</a>
                    </div>
                    <div class="divide-y divide-base-300 border-y border-base-300">
                        <For
                            each=move || posts.clone()
                            key=|post| post.title.clone()
                            children=move |post| view! { <PostRow post/> }
                        />
                    </div>
                </section>
                <aside class="space-y-4">
                    <SidebarCard title="分类" items=vec![
                        ("Leptos", "1.2k"),
                        ("教程", "856"),
                        ("实践", "642"),
                        ("前端", "512"),
                        ("后端", "398"),
                    ]/>
                    <TagCloud/>
                    <AnnouncementList/>
                    <ActiveAuthors/>
                </aside>
            </div>
        </PageShell>
    }
}

#[component]
fn PostRow(post: PostSummary) -> impl IntoView {
    view! {
        <article class="grid gap-5 py-6 md:grid-cols-[72px_1fr_108px]">
            <div class="avatar">
                <div class="h-14 w-14 rounded-lg bg-primary/10 text-primary">
                    <span class="grid h-full place-items-center text-xl font-bold">
                        {post.author_name.chars().next().unwrap_or('P').to_string()}
                    </span>
                </div>
            </div>
            <div class="min-w-0">
                <a class="link-hover text-xl font-semibold text-base-content" href="/posts/sample">
                    {post.title}
                </a>
                <p class="mt-2 line-clamp-2 text-sm leading-6 text-base-content/70">{post.summary}</p>
                <div class="mt-4 flex flex-wrap items-center gap-3 text-sm text-base-content/60">
                    <span class="font-medium text-base-content">{post.author_name}</span>
                    <span>"·"</span>
                    <span>"2 小时前"</span>
                    <span>"·"</span>
                    <span class="text-primary">{post.category_name.unwrap_or_else(|| "综合".to_string())}</span>
                    <div class="ml-auto flex flex-wrap gap-2">
                        {post.tags.into_iter().map(|tag| view! {
                            <span class="badge badge-ghost rounded-md">{tag}</span>
                        }).collect_view()}
                    </div>
                </div>
            </div>
            <div class="grid content-center gap-2 border-l border-base-300 pl-5 text-sm text-base-content/70">
                <span>"👁 " {post.view_count}</span>
                <span>"💬 " {post.comment_count}</span>
                <span>"👍 " {post.like_count}</span>
                <span>"☆ " {post.favorite_count}</span>
            </div>
        </article>
    }
}

#[component]
fn SidebarCard(title: &'static str, items: Vec<(&'static str, &'static str)>) -> impl IntoView {
    view! {
        <section class="rounded-lg border border-base-300 bg-base-100 p-5">
            <div class="mb-4 flex items-center justify-between">
                <h2 class="text-base font-semibold">{title}</h2>
                <a class="link text-sm text-base-content/60" href="/">"更多 ›"</a>
            </div>
            <ul class="space-y-3">
                {items.into_iter().map(|(name, count)| view! {
                    <li class="flex items-center justify-between text-sm">
                        <span>{name}</span>
                        <span class="text-base-content/50">{count}</span>
                    </li>
                }).collect_view()}
            </ul>
        </section>
    }
}

#[component]
fn TagCloud() -> impl IntoView {
    let tags = ["leptos", "rust", "axum", "web", "tailwindcss", "sqlx", "wasm", "server-actions"];

    view! {
        <section class="rounded-lg border border-base-300 bg-base-100 p-5">
            <h2 class="mb-4 text-base font-semibold">"热门标签"</h2>
            <div class="flex flex-wrap gap-2">
                {tags.into_iter().map(|tag| view! {
                    <span class="badge badge-ghost rounded-md px-3 py-3">{tag}</span>
                }).collect_view()}
            </div>
        </section>
    }
}

#[component]
fn AnnouncementList() -> impl IntoView {
    view! {
        <section class="rounded-lg border border-base-300 bg-base-100 p-5">
            <h2 class="mb-4 text-base font-semibold">"公告"</h2>
            <ul class="space-y-3 text-sm">
                <li class="flex justify-between gap-3"><span>"Leptos 0.8 论坛系统上线"</span><time>"2026-06-10"</time></li>
                <li class="flex justify-between gap-3"><span>"论坛规则与发帖指南"</span><time>"2026-06-09"</time></li>
                <li class="flex justify-between gap-3"><span>"关于禁止广告内容的说明"</span><time>"2026-06-01"</time></li>
            </ul>
        </section>
    }
}

#[component]
fn ActiveAuthors() -> impl IntoView {
    let authors = ["tangzx", "hello-rust", "Skyline", "CodeMika", "张开发", "Rains"];

    view! {
        <section class="rounded-lg border border-base-300 bg-base-100 p-5">
            <h2 class="mb-4 text-base font-semibold">"活跃作者"</h2>
            <div class="grid grid-cols-3 gap-4">
                {authors.into_iter().map(|author| view! {
                    <div class="text-center text-xs">
                        <div class="mx-auto mb-2 grid h-10 w-10 place-items-center rounded-lg bg-base-200 font-semibold text-primary">
                            {author.chars().next().unwrap_or('P').to_string()}
                        </div>
                        <span>{author}</span>
                    </div>
                }).collect_view()}
            </div>
        </section>
    }
}
