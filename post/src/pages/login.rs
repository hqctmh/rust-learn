use leptos::prelude::*;

use crate::components::PageShell;

#[component]
pub fn LoginPage() -> impl IntoView {
    view! {
        <PageShell>
            <section class="mx-auto max-w-md rounded-lg border border-base-300 p-6">
                <h1 class="text-2xl font-bold">"登录 Post Forum"</h1>
                <div class="mt-6 space-y-4">
                    <input class="input input-bordered w-full" placeholder="用户名"/>
                    <input class="input input-bordered w-full" type="password" placeholder="密码"/>
                    <button class="btn btn-primary w-full">"登录"</button>
                    <button class="btn btn-ghost w-full">"注册新账号"</button>
                </div>
            </section>
        </PageShell>
    }
}
