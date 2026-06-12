use leptos::prelude::*;
use leptos_router::hooks::use_params_map;

use crate::{
    components::PageShell,
    domain::{posts::PostSummary, users::UserSpace},
    page_data::{fallback_user_space_page, load_user_space_page},
};

#[component]
pub fn UserProfilePage() -> impl IntoView {
    let params = use_params_map();
    let fallback_params = params;
    let suspense_params = params;
    let space = Resource::new(
        move || user_id_from_params(&params.read(), false),
        |user_id| load_user_space_page(user_id, None),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! { <UserSpaceView space=fallback_user_space_page() route_is_me=false/> }>
                {move || {
                    let route_is_me = false;
                    let _user_id = user_id_from_params(&suspense_params.read(), route_is_me);
                    let _fallback_user_id = user_id_from_params(&fallback_params.read(), route_is_me);
                    Suspend::new(async move {
                        let space = space.await.unwrap_or_else(|_| fallback_user_space_page());
                        view! { <UserSpaceView space route_is_me/> }
                    })
                }}
            </Suspense>
        </PageShell>
    }
}

#[component]
pub fn MePage() -> impl IntoView {
    let space = Resource::new(
        || "me".to_string(),
        |user_id| load_user_space_page(user_id, None),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! { <UserSpaceView space=fallback_user_space_page() route_is_me=true/> }>
                {move || Suspend::new(async move {
                    let space = space.await.unwrap_or_else(|_| fallback_user_space_page());
                    view! { <UserSpaceView space route_is_me=true/> }
                })}
            </Suspense>
        </PageShell>
    }
}

fn user_id_from_params(params: &leptos_router::params::ParamsMap, is_me: bool) -> String {
    if is_me {
        "me".to_string()
    } else {
        params.get("id").unwrap_or_default()
    }
}

#[component]
fn UserSpaceView(space: UserSpace, route_is_me: bool) -> impl IntoView {
    let profile = space.profile.clone();
    let stats = space.stats.clone();
    let posts = if route_is_me {
        space
            .published_posts
            .iter()
            .chain(space.draft_posts.iter())
            .chain(space.favorite_posts.iter())
            .cloned()
            .collect::<Vec<_>>()
    } else {
        space.published_posts.clone()
    };
    let is_me = route_is_me;
    let follow_label = if space.followed_by_viewer {
        "取消关注"
    } else {
        "关注用户"
    };

    view! {
        <div class="user-space">
            <section class="user-hero">
                <div class="avatar-xl">{profile.nickname.chars().next().unwrap_or('P').to_string()}</div>
                <div>
                    <div class="page-kicker">{if is_me { "个人中心" } else { "用户主页" }}</div>
                    <h1>{profile.nickname.clone()}</h1>
                    <p>{profile.bio.clone()}</p>
                    <div class="profile-actions">
                        {if is_me {
                            view! {
                                <div class="profile-action-row">
                                    <button class="btn btn-primary btn-sm">"修改昵称"</button>
                                    <button class="btn btn-outline btn-sm">"修改简介"</button>
                                    <button class="btn btn-outline btn-sm">"修改头像"</button>
                                    <button class="btn btn-ghost btn-sm">"修改密码"</button>
                                </div>
                            }.into_any()
                        } else {
                            view! { <button class="btn btn-outline btn-sm">{follow_label}</button> }.into_any()
                        }}
                        <button class="btn btn-ghost btn-sm">"发送消息"</button>
                    </div>
                </div>
            </section>

            <section class="user-stat-grid">
                <UserStat label="关注" value=stats.following.to_string()/>
                <UserStat label="粉丝" value=stats.followers.to_string()/>
                <UserStat label="发布帖子" value=stats.published_posts.to_string()/>
                <UserStat label="获得喜欢" value=stats.received_likes.to_string()/>
                <UserStat label="获得收藏" value=stats.received_favorites.to_string()/>
            </section>

            <div class="user-space-grid">
                <section class="panel-card">
                    <h2>{if is_me { "我的帖子" } else { "发布的帖子" }}</h2>
                    <div class="user-post-list">
                        {if posts.is_empty() {
                            view! { <p class="muted-copy">"暂无帖子"</p> }.into_any()
                        } else {
                            view! {
                                <>
                                    {posts.into_iter().map(|post| view! { <UserPostRow post/> }).collect_view()}
                                </>
                            }.into_any()
                        }}
                    </div>
                </section>
                <section class="panel-card">
                    <h2>"个人功能"</h2>
                    <div class="system-grid">
                        <a href="/me/posts">"我的帖子"<small>{space.published_posts.len().to_string()}</small></a>
                        <a href="/me/drafts">"我的草稿"<small>{space.draft_posts.len().to_string()}</small></a>
                        <a href="/me/comments">"我的评论"<small>{space.comments.len().to_string()}</small></a>
                        <a href="/me/favorites">"我的收藏"<small>{space.favorite_posts.len().to_string()}</small></a>
                        <a href="/me/following">"我的关注"<small>{space.following.len().to_string()}</small></a>
                        <a href="/me/followers">"我的粉丝"<small>{space.followers.len().to_string()}</small></a>
                        <a href="/notifications">"消息中心"<small>"未读 / 已读"</small></a>
                    </div>
                </section>
            </div>
        </div>
    }
}

#[component]
fn UserPostRow(post: PostSummary) -> impl IntoView {
    view! {
        <a href=format!("/posts/{}", post.post_id) class="user-post-row">
            <strong>{post.title}</strong>
            <span>{post.summary}</span>
        </a>
    }
}

#[component]
fn UserStat(label: &'static str, value: String) -> impl IntoView {
    view! {
        <section class="stat-card">
            <span>{label}</span>
            <strong>{value}</strong>
            <small>"当前数据"</small>
        </section>
    }
}
