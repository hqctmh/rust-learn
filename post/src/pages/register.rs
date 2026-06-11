use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn RegisterPage() -> impl IntoView {
    view! {
        <PageShell>
            <section class="auth-layout">
                <div class="auth-panel">
                    <div class="page-kicker">"账号中心"</div>
                    <h1>"注册 Post Forum"</h1>
                    <p>"创建账号后可以发布帖子、参与评论、收藏内容、关注作者，并接收站内通知。"</p>
                    <div class="mt-6 space-y-4">
                        <input class="input input-bordered w-full" placeholder="用户名"/>
                        <input class="input input-bordered w-full" placeholder="昵称"/>
                        <input class="input input-bordered w-full" type="password" placeholder="密码"/>
                        <input class="input input-bordered w-full" type="password" placeholder="确认密码"/>
                        <button class="btn btn-primary w-full">"注册账号"</button>
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
