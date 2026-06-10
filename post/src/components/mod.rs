use leptos::prelude::*;

#[component]
pub fn TopNav() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-20 border-b border-base-300 bg-base-100/95 backdrop-blur">
            <div class="navbar mx-auto max-w-[1440px] gap-4 px-6">
                <a class="btn btn-ghost gap-2 text-xl font-bold" href="/">
                    <span class="rounded-md bg-primary px-2 py-1 text-primary-content">"</>"</span>
                    "Post Forum"
                </a>
                <nav class="hidden gap-1 lg:flex">
                    <a class="btn btn-ghost btn-sm text-primary" href="/">"首页"</a>
                    <a class="btn btn-ghost btn-sm" href="/?tab=discover">"发现"</a>
                    <a class="btn btn-ghost btn-sm" href="/?tab=docs">"文档"</a>
                </nav>
                <label class="input input-bordered ml-auto hidden w-full max-w-md items-center gap-2 md:flex">
                    <span class="text-base-content/50">"⌕"</span>
                    <input type="search" class="grow" placeholder="搜索帖子、标签、用户..."/>
                    <kbd class="kbd kbd-sm">"/"</kbd>
                </label>
                <a class="btn btn-primary btn-sm" href="/posts/new">"发布帖子"</a>
                <a class="btn btn-success btn-sm text-success-content" href="/admin">"管理端"</a>
                <a class="btn btn-ghost btn-circle btn-sm" href="/login" aria-label="通知">"🔔"</a>
                <a class="btn btn-ghost btn-sm" href="/login">"登录"</a>
            </div>
        </header>
    }
}

#[component]
pub fn PageShell(children: Children) -> impl IntoView {
    view! {
        <div class="min-h-screen bg-base-100 text-base-content">
            <TopNav/>
            <main class="mx-auto max-w-[1440px] px-6 py-7">
                {children()}
            </main>
        </div>
    }
}
