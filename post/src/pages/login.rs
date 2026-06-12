use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::components::PageShell;
use crate::page_data::LoginUser;

#[component]
pub fn LoginPage() -> impl IntoView {
    let login_action = ServerAction::<LoginUser>::new();
    let action = login_action;
    let login_pending = action.pending();
    let login_result = action.value();

    view! {
        <PageShell>
            <section class="auth-layout">
                <div class="auth-panel">
                    <div class="page-kicker">"账号中心"</div>
                    <h1>"登录 Post Forum"</h1>
                    <p>"登录后可以发布帖子、评论回复、点赞收藏、关注作者，并接收 WebSocket 实时通知。"</p>
                    <div class="mt-6 space-y-4">
                        <ActionForm action=login_action>
                            <div class="auth-form-fields">
                                <input class="input input-bordered w-full" name="username" placeholder="用户名或邮箱" autocomplete="username"/>
                                <input class="input input-bordered w-full" name="password" type="password" placeholder="密码" autocomplete="current-password"/>
                                <button class="btn btn-primary w-full" type="submit" disabled=move || login_pending.get()>
                                    {move || if login_pending.get() { "登录中..." } else { "登录" }}
                                </button>
                            </div>
                        </ActionForm>
                        {move || {
                            login_result.get().map(|result| match result {
                                Ok(session) => view! {
                                    <div class="auth-feedback auth-feedback-success">
                                        <strong>"登录成功"</strong>
                                        <span>"当前会话 ID"</span>
                                        <code>{session.session_id.to_string()}</code>
                                        <a href=format!("/posts/new?session_id={}", session.session_id)>"去发布帖子"</a>
                                    </div>
                                }.into_any(),
                                Err(error) => view! {
                                    <div class="auth-feedback auth-feedback-error">
                                        <strong>"登录失败"</strong>
                                        <span>{error.to_string()}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                        <a class="btn btn-outline w-full" href="/register">"注册新账号"</a>
                    </div>
                </div>
                <aside class="panel-card account-panel">
                    <h2>"个人主页能力"</h2>
                    <div class="system-grid">
                        <a href="/me">"修改头像"<small>"上传到 RustFS"</small></a>
                        <a href="/me">"修改资料"<small>"昵称 / 简介 / 密码"</small></a>
                        <a href="/me">"我的草稿"<small>"自动保存"</small></a>
                        <a href="/me">"我的收藏"<small>"收藏列表"</small></a>
                        <a href="/me">"关注与粉丝"<small>"关注列表"</small></a>
                        <a href="/notifications">"消息中心"<small>"全部已读"</small></a>
                    </div>
                </aside>
            </section>
        </PageShell>
    }
}
