use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{
    components::PageShell,
    domain::comments::CommentNode,
    page_data::{PostDetailPageData, fallback_post_detail_page, load_post_detail_page},
};

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let fallback_params = params;
    let suspense_params = params;
    let data = Resource::new(
        move || post_id_from_params(&params.read()),
        load_post_detail_page,
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <PostDetailView data=fallback_post_detail_page(post_id_from_params(&fallback_params.read()))/>
            }>
                {move || {
                    let post_id = post_id_from_params(&suspense_params.read());
                    Suspend::new(async move {
                        let data = data
                            .await
                            .unwrap_or_else(|_| fallback_post_detail_page(post_id.clone()));
                        view! { <PostDetailView data/> }
                    })
                }}
            </Suspense>
        </PageShell>
    }
}

fn post_id_from_params(params: &leptos_router::params::ParamsMap) -> String {
    params.get("id").unwrap_or_default()
}

#[component]
fn PostDetailView(data: PostDetailPageData) -> impl IntoView {
    let post = data.post.clone();
    let summary = post.summary.clone();
    let category = summary
        .category_name
        .clone()
        .unwrap_or_else(|| "未分类".to_string());
    let tags = summary.tags.clone();
    let comments = data.comments.clone();
    let comment_count = comments.len();

    view! {
        <div class="detail-layout">
            <article class="detail-article">
                <div class="breadcrumb">"首页 / " {category.clone()} " / " {tags.first().cloned().unwrap_or_else(|| "帖子".to_string())}</div>
                <h1>{summary.title.clone()}</h1>
                <div class="detail-meta">
                    <span class="avatar-mini">{summary.author_name.chars().next().unwrap_or('P').to_string()}</span>
                    <span>{summary.author_name.clone()}</span>
                    <span>{summary.published_at.map(|time| format!("发布于 {time}")).unwrap_or_else(|| "未发布".to_string())}</span>
                    <span class=format!("badge badge-{}", category_color(&category))>{category.clone()}</span>
                    {tags.into_iter().map(|tag| view! {
                        <span class="badge badge-soft">{tag}</span>
                    }).collect_view()}
                </div>
                <div class="article-stats">
                    <span>"浏览 " {compact_count(summary.view_count)}</span>
                    <span>"点赞 " {summary.like_count}</span>
                    <span>"收藏 " {summary.favorite_count}</span>
                    <span>"评论 " {summary.comment_count}</span>
                </div>
                <div class="prose article-body" inner_html=post.sanitized_html.clone()></div>
                <div class="detail-actions">
                    <button class=if post.liked_by_me { "btn btn-primary" } else { "btn btn-outline" }>"点赞 " {summary.like_count}</button>
                    <button class=if post.favorited_by_me { "btn btn-primary" } else { "btn btn-outline" }>"收藏 " {summary.favorite_count}</button>
                    <button class=if post.following_author { "btn btn-primary" } else { "btn btn-outline" }>"关注作者"</button>
                    <button class="btn btn-ghost">"举报"</button>
                </div>
                <section class="comment-panel">
                    <div class="section-heading">
                        <h2>"评论与回复"</h2>
                        <a href="/login">"登录后互动"</a>
                    </div>
                    <textarea class="textarea textarea-bordered w-full" placeholder="写下你的评论，支持回复评论"></textarea>
                    <div class="comment-actions"><button class="btn btn-primary btn-sm">"发表评论"</button></div>
                    {if comment_count == 0 {
                        view! { <p class="muted-copy">"暂无评论，成为第一个参与讨论的人。"</p> }.into_any()
                    } else {
                        view! {
                            <>
                                {comments.into_iter().map(|comment| view! { <CommentItem comment/> }).collect_view()}
                            </>
                        }.into_any()
                    }}
                </section>
            </article>
            <aside class="side-stack">
                <section class="panel-card">
                    <h2>"作者信息"</h2>
                    <div class="profile-card">
                        <span class="avatar-lg">{summary.author_name.chars().next().unwrap_or('P').to_string()}</span>
                        <strong>{summary.author_name.clone()}</strong>
                        <p>{summary.author_avatar_url.clone().unwrap_or_else(|| "论坛成员".to_string())}</p>
                        <div class="profile-stats"><span>"点赞 " {summary.like_count}</span><span>"收藏 " {summary.favorite_count}</span></div>
                    </div>
                </section>
                <section class="panel-card">
                    <h2>"相关推荐"</h2>
                    <ul class="related-list">
                        <li><a href="/search?q=server%20function">"server function 中使用 SQLx 事务"</a><small>"7 回复"</small></li>
                        <li><a href="/search?q=markdown">"Markdown 渲染与代码高亮"</a><small>"3 回复"</small></li>
                        <li><a href="/search?q=axum">"Axum 中间件处理 request body"</a><small>"1 回复"</small></li>
                    </ul>
                </section>
                <section class="panel-card">
                    <h2>"权限提示"</h2>
                    <p class="muted-copy">"未登录用户可以查看内容和评论；点赞、收藏、关注、评论、回复和举报会引导登录。"</p>
                </section>
            </aside>
        </div>
    }
}

#[component]
fn CommentItem(comment: CommentNode) -> impl IntoView {
    let replies = comment.replies.clone();
    let badge = if comment.author_reply {
        "作者回复".to_string()
    } else if replies.is_empty() {
        "评论".to_string()
    } else {
        format!("{} 条回复", replies.len())
    };

    view! {
        <div class="comment-item">
            <span class="avatar-mini">{comment.author_name.chars().next().unwrap_or('P').to_string()}</span>
            <div>
                <div class="comment-meta"><strong>{comment.author_name}</strong><span class="badge badge-soft">{badge}</span></div>
                <p>{comment.content}</p>
                <div class="comment-toolrow">
                    <button class="btn btn-ghost btn-xs">"回复评论"</button>
                    <button class="btn btn-ghost btn-xs">"点赞评论 " {comment.like_count}</button>
                    <button class="btn btn-ghost btn-xs">"删除自己的评论"</button>
                    <button class="btn btn-ghost btn-xs">"举报评论"</button>
                </div>
                {if replies.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <div class="comment-replies">
                            {replies.into_iter().map(|reply| view! { <CommentItem comment=reply/> }).collect_view()}
                        </div>
                    }.into_any()
                }}
            </div>
        </div>
    }
}

fn compact_count(count: i64) -> String {
    if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.max(0).to_string()
    }
}

fn category_color(category: &str) -> &'static str {
    match category {
        "公告" => "blue",
        "教程" => "green",
        "问题" => "orange",
        "经验分享" => "sky",
        "站务" => "purple",
        _ => "gray",
    }
}
