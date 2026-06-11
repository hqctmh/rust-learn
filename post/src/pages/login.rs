use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <PageShell>
            <section class="auth-layout">
                <div class="auth-panel">
                    <div class="page-kicker">"账号中心"</div>
                    <h1>"登录 Post Forum"</h1>
                    <p>"登录后可以发布帖子、评论回复、点赞收藏、关注作者，并接收 WebSocket 实时通知。"</p>
                    <div class="mt-6 space-y-4">
                        <input class="input input-bordered w-full" placeholder="用户名或邮箱"/>
                        <input class="input input-bordered w-full" type="password" placeholder="密码"/>
                        <button class="btn btn-primary w-full">"登录"</button>
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
