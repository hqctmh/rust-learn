use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::{
        posts::PostSummary,
        users::{UserProfile, UserStats},
    },
};

#[component]
pub fn UserProfilePage() -> impl IntoView {
    view! {
        <PageShell>
            <UserSpaceView is_me=false/>
        </PageShell>
    }
}

#[component]
pub fn MePage() -> impl IntoView {
    view! {
        <PageShell>
            <UserSpaceView is_me=true/>
        </PageShell>
    }
}

#[component]
fn UserSpaceView(is_me: bool) -> impl IntoView {
    let profile = demo_profile();
    let stats = demo_stats();
    let posts = demo_posts();

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
                            view! { <button class="btn btn-outline btn-sm">"关注用户"</button> }.into_any()
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
                        {posts.into_iter().map(|post| view! {
                            <a href=format!("/posts/{}", post.post_id) class="user-post-row">
                                <strong>{post.title}</strong>
                                <span>{post.summary}</span>
                            </a>
                        }).collect_view()}
                    </div>
                </section>
                <section class="panel-card">
                    <h2>"个人功能"</h2>
                    <div class="system-grid">
                        <a href="/me/posts">"我的帖子"<small>"已发布"</small></a>
                        <a href="/me/drafts">"我的草稿"<small>"自动保存"</small></a>
                        <a href="/me/comments">"我的评论"<small>"评论历史"</small></a>
                        <a href="/me/favorites">"我的收藏"<small>"收藏列表"</small></a>
                        <a href="/me/following">"我的关注"<small>"关注用户"</small></a>
                        <a href="/me/followers">"我的粉丝"<small>"粉丝列表"</small></a>
                        <a href="/notifications">"消息中心"<small>"未读 / 已读"</small></a>
                    </div>
                </section>
            </div>
        </div>
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

fn demo_profile() -> UserProfile {
    UserProfile {
        user_id: uuid::Uuid::from_u128(1),
        username: "mah".to_string(),
        nickname: "mah".to_string(),
        avatar_url: None,
        bio: "Post Forum 管理员，关注 Leptos、Axum 与 SQLx。".to_string(),
        registered_at: time::OffsetDateTime::now_utc(),
    }
}

fn demo_stats() -> UserStats {
    UserStats {
        following: 12,
        followers: 248,
        published_posts: 18,
        received_likes: 920,
        received_favorites: 316,
    }
}

fn demo_posts() -> Vec<PostSummary> {
    vec![PostSummary::sample()]
}
