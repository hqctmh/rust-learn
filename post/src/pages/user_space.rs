use leptos::form::ActionForm;
use leptos::prelude::*;
use leptos_router::hooks::{use_params_map, use_query_map};

use crate::{
    components::PageShell,
    domain::{posts::PostSummary, users::UserSpace},
    page_data::{
        ChangeMePassword, ToggleAuthorFollow, UpdateMeAvatar, UpdateMeProfile,
        fallback_user_space_page, load_user_space_page,
    },
};

#[component]
pub fn UserProfilePage() -> impl IntoView {
    let params = use_params_map();
    let query = use_query_map();
    let fallback_params = params;
    let suspense_params = params;
    let fallback_query = query;
    let suspense_query = query;
    let space = Resource::new(
        move || {
            (
                user_id_from_params(&params.read(), false),
                query.read().get("session_id").unwrap_or_default(),
            )
        },
        |(user_id, session_id)| load_user_space_page(user_id, Some(session_id)),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <UserSpaceView
                    space=fallback_user_space_page()
                    route_is_me=false
                    session_id=fallback_query.read().get("session_id").unwrap_or_default()
                />
            }>
                {move || {
                    let route_is_me = false;
                    let _user_id = user_id_from_params(&suspense_params.read(), route_is_me);
                    let _fallback_user_id = user_id_from_params(&fallback_params.read(), route_is_me);
                    let session_id = suspense_query.read().get("session_id").unwrap_or_default();
                    Suspend::new(async move {
                        let space = space.await.unwrap_or_else(|_| fallback_user_space_page());
                        view! { <UserSpaceView space route_is_me session_id/> }
                    })
                }}
            </Suspense>
        </PageShell>
    }
}

#[component]
pub fn MePage() -> impl IntoView {
    let query = use_query_map();
    let fallback_query = query;
    let suspense_query = query;
    let space = Resource::new(
        move || query.read().get("session_id").unwrap_or_default(),
        |session_id| load_user_space_page("me".to_string(), Some(session_id)),
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <UserSpaceView
                    space=fallback_user_space_page()
                    route_is_me=true
                    session_id=fallback_query.read().get("session_id").unwrap_or_default()
                />
            }>
                {move || Suspend::new(async move {
                    let space = space.await.unwrap_or_else(|_| fallback_user_space_page());
                    let session_id = suspense_query.read().get("session_id").unwrap_or_default();
                    view! { <UserSpaceView space route_is_me=true session_id/> }
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
fn UserSpaceView(space: UserSpace, route_is_me: bool, session_id: String) -> impl IntoView {
    let profile_action = ServerAction::<UpdateMeProfile>::new();
    let avatar_action = ServerAction::<UpdateMeAvatar>::new();
    let password_action = ServerAction::<ChangeMePassword>::new();
    let follow_action = ServerAction::<ToggleAuthorFollow>::new();
    let profile_pending = profile_action.pending();
    let avatar_pending = avatar_action.pending();
    let password_pending = password_action.pending();
    let follow_pending = follow_action.pending();
    let profile_result = profile_action.value();
    let avatar_result = avatar_action.value();
    let password_result = password_action.value();
    let follow_result = follow_action.value();
    let initial_profile = space.profile.clone();
    let profile = Memo::new(move |_| {
        if let Some(Ok(profile)) = avatar_result.get() {
            return profile;
        }
        if let Some(Ok(profile)) = profile_result.get() {
            return profile;
        }
        initial_profile.clone()
    });
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
    let initial_followed = space.followed_by_viewer;
    let current_following = Memo::new(move |_| {
        follow_result
            .get()
            .and_then(Result::ok)
            .map(|state| state.following)
            .unwrap_or(initial_followed)
    });
    let follow_label = if initial_followed {
        "取消关注"
    } else {
        "关注用户"
    };
    let settings_disabled_session_id = session_id.clone();
    let profile_session_id = session_id.clone();
    let avatar_session_id = session_id.clone();
    let avatar_disabled_session_id = session_id.clone();
    let password_session_id = session_id.clone();
    let password_disabled_session_id = session_id.clone();
    let follow_session_id = session_id.clone();
    let follow_disabled_session_id = session_id.clone();

    view! {
        <div class="user-space">
            <section class="user-hero">
                <div class="avatar-xl">{move || profile.get().nickname.chars().next().unwrap_or('P').to_string()}</div>
                <div>
                    <div class="page-kicker">{if is_me { "个人中心" } else { "用户主页" }}</div>
                    <h1>{move || profile.get().nickname}</h1>
                    <p>{move || profile.get().bio}</p>
                    <div class="profile-actions">
                        {if is_me {
                            view! {
                                <div class="profile-action-stack">
                                    <ActionForm action=profile_action>
                                        <input type="hidden" name="session_id" value=profile_session_id/>
                                        <input
                                            class="input input-bordered input-sm"
                                            name="nickname"
                                            value=move || profile.get().nickname
                                            placeholder="昵称"
                                        />
                                        <input
                                            class="input input-bordered input-sm"
                                            name="bio"
                                            value=move || profile.get().bio
                                            placeholder="个人简介"
                                        />
                                        <button
                                            class="btn btn-primary btn-sm"
                                            type="submit"
                                            disabled=move || profile_pending.get() || settings_disabled_session_id.is_empty()
                                        >
                                            "修改昵称和简介"
                                        </button>
                                    </ActionForm>
                                    <ActionForm action=avatar_action>
                                        <input type="hidden" name="session_id" value=avatar_session_id/>
                                        <input
                                            class="input input-bordered input-sm"
                                            name="avatar_url"
                                            value=move || profile.get().avatar_url.unwrap_or_default()
                                            placeholder="/uploads/avatars/me.png"
                                        />
                                        <button
                                            class="btn btn-outline btn-sm"
                                            type="submit"
                                            disabled=move || avatar_pending.get() || avatar_disabled_session_id.is_empty()
                                        >
                                            "修改头像"
                                        </button>
                                    </ActionForm>
                                    <ActionForm action=password_action>
                                        <input type="hidden" name="session_id" value=password_session_id/>
                                        <input class="input input-bordered input-sm" name="old_password" type="password" placeholder="旧密码"/>
                                        <input class="input input-bordered input-sm" name="new_password" type="password" placeholder="新密码"/>
                                        <button
                                            class="btn btn-ghost btn-sm"
                                            type="submit"
                                            disabled=move || password_pending.get() || password_disabled_session_id.is_empty()
                                        >
                                            "修改密码"
                                        </button>
                                    </ActionForm>
                                    <div class="profile-action-feedback">
                                        {move || {
                                            if let Some(result) = profile_result.get() {
                                                return match result {
                                                    Ok(profile) => view! { <p class="success"><strong>"资料更新成功"</strong>{profile.nickname}</p> }.into_any(),
                                                    Err(error) => view! { <p class="error"><strong>"资料更新失败"</strong>{error.to_string()}</p> }.into_any(),
                                                };
                                            }
                                            if let Some(result) = avatar_result.get() {
                                                return match result {
                                                    Ok(profile) => view! { <p class="success"><strong>"头像更新成功"</strong>{profile.avatar_url.unwrap_or_default()}</p> }.into_any(),
                                                    Err(error) => view! { <p class="error"><strong>"头像更新失败"</strong>{error.to_string()}</p> }.into_any(),
                                                };
                                            }
                                            if let Some(result) = password_result.get() {
                                                return match result {
                                                    Ok(message) => view! { <p class="success"><strong>"密码更新成功"</strong>{message}</p> }.into_any(),
                                                    Err(error) => view! { <p class="error"><strong>"密码更新失败"</strong>{error.to_string()}</p> }.into_any(),
                                                };
                                            }
                                            ().into_any()
                                        }}
                                    </div>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="profile-action-stack compact">
                                    <ActionForm action=follow_action>
                                        <input type="hidden" name="session_id" value=follow_session_id/>
                                        <input
                                            type="hidden"
                                            name="author_id"
                                            value=move || profile.get().user_id.to_string()
                                        />
                                        <button
                                            class="btn btn-outline btn-sm"
                                            type="submit"
                                            disabled=move || follow_pending.get() || follow_disabled_session_id.is_empty()
                                        >
                                            {move || if current_following.get() { "取消关注" } else { follow_label }}
                                        </button>
                                    </ActionForm>
                                    <div class="profile-action-feedback">
                                        {move || {
                                            if let Some(result) = follow_result.get() {
                                                return match result {
                                                    Ok(state) => view! {
                                                        <p class="success">
                                                            <strong>"关注成功"</strong>
                                                            {if state.following { "已关注该用户" } else { "已取消关注" }}
                                                        </p>
                                                    }.into_any(),
                                                    Err(error) => view! {
                                                        <p class="error"><strong>"关注失败"</strong>{error.to_string()}</p>
                                                    }.into_any(),
                                                };
                                            }
                                            ().into_any()
                                        }}
                                    </div>
                                </div>
                            }.into_any()
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
