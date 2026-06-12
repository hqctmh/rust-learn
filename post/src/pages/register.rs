use leptos::form::ActionForm;
use leptos::prelude::*;

use crate::components::PageShell;
use crate::page_data::RegisterUser;

#[component]
pub fn RegisterPage() -> impl IntoView {
    let register_action = ServerAction::<RegisterUser>::new();
    let action = register_action;
    let register_pending = action.pending();
    let register_result = action.value();

    view! {
        <PageShell>
            <section class="auth-layout">
                <div class="auth-panel">
                    <div class="page-kicker">"账号中心"</div>
                    <h1>"注册 Post Forum"</h1>
                    <p>"创建账号后可以发布帖子、参与评论、收藏内容、关注作者，并接收站内通知。"</p>
                    <div class="mt-6 space-y-4">
                        <ActionForm action=register_action>
                            <div class="auth-form-fields">
                                <input class="input input-bordered w-full" name="username" placeholder="用户名" autocomplete="username"/>
                                <input class="input input-bordered w-full" name="nickname" placeholder="昵称" autocomplete="name"/>
                                <input class="input input-bordered w-full" name="password" type="password" placeholder="密码" autocomplete="new-password"/>
                                <input class="input input-bordered w-full" name="confirm_password" type="password" placeholder="确认密码" autocomplete="new-password"/>
                                <button class="btn btn-primary w-full" type="submit" disabled=move || register_pending.get()>
                                    {move || if register_pending.get() { "注册中..." } else { "注册账号" }}
                                </button>
                            </div>
                        </ActionForm>
                        {move || {
                            register_result.get().map(|result| match result {
                                Ok(session) => view! {
                                    <div class="auth-feedback auth-feedback-success">
                                        <strong>"注册成功"</strong>
                                        <span>"当前会话 ID"</span>
                                        <code>{session.session_id.to_string()}</code>
                                        <a href=format!("/posts/new?session_id={}", session.session_id)>"去发布帖子"</a>
                                    </div>
                                }.into_any(),
                                Err(error) => view! {
                                    <div class="auth-feedback auth-feedback-error">
                                        <strong>"注册失败"</strong>
                                        <span>{error.to_string()}</span>
                                    </div>
                                }.into_any(),
                            })
                        }}
                        <a class="btn btn-outline w-full" href="/login">"已有账号，去登录"</a>
                    </div>
                </div>
                <aside class="panel-card account-panel">
                    <h2>"注册后可用能力"</h2>
                    <div class="system-grid">
                        <a href="/posts/new">"发布帖子"<small>"Markdown 编辑 / 图片上传"</small></a>
                        <a href="/me">"个人中心"<small>"资料 / 草稿 / 收藏"</small></a>
                        <a href="/notifications">"消息通知"<small>"评论 / 点赞 / 公告"</small></a>
                        <a href="/">"关注动态"<small>"查看关注作者内容"</small></a>
                    </div>
                </aside>
            </section>
        </PageShell>
    }
}
