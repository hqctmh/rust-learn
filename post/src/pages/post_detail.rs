use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::{
    components::PageShell,
    domain::comments::CommentNode,
    page_data::{
        DeleteOwnComment, PostDetailPageData, ReportComment, ReportPost, SubmitComment,
        ToggleAuthorFollow, ToggleCommentLike, TogglePostFavorite, TogglePostLike,
        fallback_post_detail_page, load_post_detail_page,
    },
};

#[component]
pub fn PostDetailPage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let fallback_params = params;
    let suspense_params = params;
    let fallback_query = query;
    let suspense_query = query;
    let data = Resource::new(
        move || post_id_from_params(&params.read()),
        load_post_detail_page,
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <PostDetailView
                    data=fallback_post_detail_page(post_id_from_params(&fallback_params.read()))
                    session_id=fallback_query.read().get("session_id").unwrap_or_default()
                />
            }>
                {move || {
                    let post_id = post_id_from_params(&suspense_params.read());
                    Suspend::new(async move {
                        let data = data
                            .await
                            .unwrap_or_else(|_| fallback_post_detail_page(post_id.clone()));
                        let session_id = suspense_query.read().get("session_id").unwrap_or_default();
                        view! { <PostDetailView data session_id/> }
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
fn PostDetailView(data: PostDetailPageData, session_id: String) -> impl IntoView {
    let comment_action = ServerAction::<SubmitComment>::new();
    let like_action = ServerAction::<TogglePostLike>::new();
    let favorite_action = ServerAction::<TogglePostFavorite>::new();
    let follow_action = ServerAction::<ToggleAuthorFollow>::new();
    let comment_like_action = ServerAction::<ToggleCommentLike>::new();
    let comment_delete_action = ServerAction::<DeleteOwnComment>::new();
    let report_post_action = ServerAction::<ReportPost>::new();
    let report_comment_action = ServerAction::<ReportComment>::new();
    let comment_pending = comment_action.pending();
    let like_pending = like_action.pending();
    let favorite_pending = favorite_action.pending();
    let follow_pending = follow_action.pending();
    let comment_result = comment_action.value();
    let like_result = like_action.value();
    let favorite_result = favorite_action.value();
    let follow_result = follow_action.value();
    let comment_like_result = comment_like_action.value();
    let comment_delete_result = comment_delete_action.value();
    let report_post_result = report_post_action.value();
    let report_comment_result = report_comment_action.value();
    let post = data.post.clone();
    let summary = post.summary.clone();
    let category = summary
        .category_name
        .clone()
        .unwrap_or_else(|| "未分类".to_string());
    let tags = summary.tags.clone();
    let comments = data.comments.clone();
    let comment_count = comments.len();
    let has_session = !session_id.is_empty();
    let post_id = summary.post_id.to_string();
    let author_id = summary.author_id.to_string();
    let like_session_id = session_id.clone();
    let favorite_session_id = session_id.clone();
    let follow_session_id = session_id.clone();
    let report_post_session_id = session_id.clone();
    let comment_session_id = session_id.clone();
    let comment_tree_session_id = session_id.clone();
    let like_post_id = post_id.clone();
    let favorite_post_id = post_id.clone();
    let report_post_id = post_id.clone();
    let comment_post_id = post_id.clone();
    let comment_tree_post_id = post_id.clone();
    let liked_by_me = post.liked_by_me;
    let favorited_by_me = post.favorited_by_me;
    let following_author = post.following_author;
    let like_count = summary.like_count;
    let favorite_count = summary.favorite_count;

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
                    <ActionForm action=like_action>
                        <input type="hidden" name="session_id" value=like_session_id/>
                        <input type="hidden" name="post_id" value=like_post_id/>
                        <button class=if liked_by_me { "btn btn-primary" } else { "btn btn-outline" } type="submit" disabled=move || like_pending.get() || !has_session>
                            {move || if like_pending.get() { "处理中..." } else if liked_by_me { "取消点赞 " } else { "点赞 " }}
                            {like_count}
                        </button>
                    </ActionForm>
                    <ActionForm action=favorite_action>
                        <input type="hidden" name="session_id" value=favorite_session_id/>
                        <input type="hidden" name="post_id" value=favorite_post_id/>
                        <button class=if favorited_by_me { "btn btn-primary" } else { "btn btn-outline" } type="submit" disabled=move || favorite_pending.get() || !has_session>
                            {move || if favorite_pending.get() { "处理中..." } else if favorited_by_me { "取消收藏 " } else { "收藏 " }}
                            {favorite_count}
                        </button>
                    </ActionForm>
                    <ActionForm action=follow_action>
                        <input type="hidden" name="session_id" value=follow_session_id/>
                        <input type="hidden" name="author_id" value=author_id.clone()/>
                        <button class=if following_author { "btn btn-primary" } else { "btn btn-outline" } type="submit" disabled=move || follow_pending.get() || !has_session>
                            {move || if follow_pending.get() { "处理中..." } else if following_author { "取消关注" } else { "关注作者" }}
                        </button>
                    </ActionForm>
                    <ActionForm action=report_post_action>
                        <input type="hidden" name="session_id" value=report_post_session_id/>
                        <input type="hidden" name="target_id" value=report_post_id/>
                        <input type="hidden" name="reason" value="内容不友好"/>
                        <input type="hidden" name="description" value="来自帖子详情页的帖子举报"/>
                        <button class="btn btn-ghost" type="submit" disabled=move || !has_session>"举报帖子"</button>
                    </ActionForm>
                </div>
                <div class="detail-action-feedback">
                    {move || {
                        like_result.get().map(|result| match result {
                            Ok(state) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"点赞成功"</strong>
                                    <span>{if state.active { "已点赞" } else { "已取消点赞" }} " · " {state.count}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"互动失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        favorite_result.get().map(|result| match result {
                            Ok(state) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"收藏成功"</strong>
                                    <span>{if state.active { "已收藏" } else { "已取消收藏" }} " · " {state.count}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"互动失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        follow_result.get().map(|result| match result {
                            Ok(state) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"关注成功"</strong>
                                    <span>{if state.following { "已关注作者" } else { "已取消关注作者" }}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"互动失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        report_post_result.get().map(|result| match result {
                            Ok(report) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"举报成功"</strong>
                                    <span>{report.reason}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"举报失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                </div>
                <section class="comment-panel">
                    <div class="section-heading">
                        <h2>"评论与回复"</h2>
                        <a href="/login">"登录后互动"</a>
                    </div>
                    <ActionForm action=comment_action>
                        <input type="hidden" name="session_id" value=comment_session_id/>
                        <input type="hidden" name="post_id" value=comment_post_id/>
                        <input type="hidden" name="parent_comment_id" value=""/>
                        <textarea class="textarea textarea-bordered w-full" name="content" placeholder="写下你的评论，支持回复评论"></textarea>
                        <div class="comment-actions">
                            <button class="btn btn-primary btn-sm" type="submit" disabled=move || comment_pending.get() || session_id.is_empty()>
                                {move || if comment_pending.get() { "发表中..." } else { "发表评论" }}
                            </button>
                        </div>
                    </ActionForm>
                    {move || {
                        comment_result.get().map(|result| match result {
                            Ok(comment) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"评论成功"</strong>
                                    <span>{comment.content}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"评论失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        comment_like_result.get().map(|result| match result {
                            Ok(state) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"评论点赞成功"</strong>
                                    <span>{if state.active { "已点赞评论" } else { "已取消评论点赞" }} " · " {state.count}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"评论操作失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        comment_delete_result.get().map(|result| match result {
                            Ok(comment) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"评论删除成功"</strong>
                                    <span>{comment.content}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"评论操作失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {move || {
                        report_comment_result.get().map(|result| match result {
                            Ok(report) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-success">
                                    <strong>"举报成功"</strong>
                                    <span>{report.reason}</span>
                                </div>
                            }.into_any(),
                            Err(error) => view! {
                                <div class="editor-feedback auth-feedback auth-feedback-error">
                                    <strong>"举报失败"</strong>
                                    <span>{error.to_string()}</span>
                                </div>
                            }.into_any(),
                        })
                    }}
                    {if comment_count == 0 {
                        view! { <p class="muted-copy">"暂无评论，成为第一个参与讨论的人。"</p> }.into_any()
                    } else {
                        view! {
                            <>
                                {comments.into_iter().map(|comment| view! {
                                    <CommentItem
                                        comment
                                        comment_action
                                        comment_like_action
                                        comment_delete_action
                                        report_comment_action
                                        session_id=comment_tree_session_id.clone()
                                        post_id=comment_tree_post_id.clone()
                                    />
                                }).collect_view()}
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
fn CommentItem(
    comment: CommentNode,
    comment_action: ServerAction<SubmitComment>,
    comment_like_action: ServerAction<ToggleCommentLike>,
    comment_delete_action: ServerAction<DeleteOwnComment>,
    report_comment_action: ServerAction<ReportComment>,
    session_id: String,
    post_id: String,
) -> impl IntoView {
    let replies = comment.replies.clone();
    let reply_session_id = session_id.clone();
    let reply_post_id = post_id.clone();
    let reply_parent_comment_id = comment.comment_id.to_string();
    let reply_submit_disabled = session_id.clone();
    let comment_like_session_id = session_id.clone();
    let comment_delete_session_id = session_id.clone();
    let comment_like_id = comment.comment_id.to_string();
    let comment_delete_id = comment.comment_id.to_string();
    let report_comment_session_id = session_id.clone();
    let report_comment_id = comment.comment_id.to_string();
    let comment_like_disabled = session_id.clone();
    let comment_delete_disabled = session_id.clone();
    let report_comment_disabled = session_id.clone();
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
                    <ActionForm action=comment_like_action>
                        <input type="hidden" name="session_id" value=comment_like_session_id/>
                        <input type="hidden" name="comment_id" value=comment_like_id/>
                        <button class="btn btn-ghost btn-xs" type="submit" disabled=move || comment_like_disabled.is_empty()>
                            "点赞评论 " {comment.like_count}
                        </button>
                    </ActionForm>
                    <ActionForm action=comment_delete_action>
                        <input type="hidden" name="session_id" value=comment_delete_session_id/>
                        <input type="hidden" name="comment_id" value=comment_delete_id/>
                        <button class="btn btn-ghost btn-xs" type="submit" disabled=move || comment_delete_disabled.is_empty()>
                            "删除自己的评论"
                        </button>
                    </ActionForm>
                    <ActionForm action=report_comment_action>
                        <input type="hidden" name="session_id" value=report_comment_session_id/>
                        <input type="hidden" name="comment_id" value=report_comment_id/>
                        <input type="hidden" name="reason" value="内容不友好"/>
                        <input type="hidden" name="description" value="来自帖子详情页的评论举报"/>
                        <button class="btn btn-ghost btn-xs" type="submit" disabled=move || report_comment_disabled.is_empty()>
                            "举报评论"
                        </button>
                    </ActionForm>
                </div>
                <ActionForm action=comment_action>
                    <input type="hidden" name="session_id" value=reply_session_id/>
                    <input type="hidden" name="post_id" value=reply_post_id/>
                    <input type="hidden" name="parent_comment_id" value=reply_parent_comment_id/>
                    <textarea class="textarea textarea-bordered w-full" name="content" placeholder="回复这条评论"></textarea>
                    <div class="comment-actions">
                        <button class="btn btn-ghost btn-xs" type="submit" disabled=move || reply_submit_disabled.is_empty()>
                            "提交回复"
                        </button>
                    </div>
                </ActionForm>
                {if replies.is_empty() {
                    view! { <></> }.into_any()
                } else {
                    view! {
                        <div class="comment-replies">
                            {replies.into_iter().map(|reply| view! { <CommentItem comment=reply comment_action comment_like_action comment_delete_action report_comment_action session_id=session_id.clone() post_id=post_id.clone()/> }).collect_view()}
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
