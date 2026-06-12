use leptos::prelude::*;

#[component]
pub fn TopNav() -> impl IntoView {
    view! {
        <header class="sticky top-0 z-20 border-b border-base-300 bg-base-100/95 backdrop-blur">
            <div class="navbar mx-auto max-w-[1440px] flex-wrap gap-4 px-5">
                <a class="brand-link" href="/">
                    <span class="brand-mark">"P"</span>
                    "Post Forum"
                </a>
                <nav class="top-tabs hidden gap-1 lg:flex">
                    <a class="top-tab active" href="/">"首页"</a>
                    <a class="top-tab" href="/?tab=posts">"帖子"</a>
                    <a class="top-tab" href="/?tab=tags">"标签"</a>
                    <a class="top-tab" href="/users/sample">"用户"</a>
                    <a class="top-tab" href="/?tab=docs">"文档"</a>
                    <a class="top-tab" href="/?tab=events">"活动"</a>
                </nav>
                <form class="search-pill ml-auto hidden min-w-0 w-full max-w-md items-center gap-2 md:flex" action="/search" method="get">
                    <span class="icon-text" aria-hidden="true">"⌕"</span>
                    <input name="q" type="search" class="grow" placeholder="搜索帖子、标签、用户..."/>
                    <kbd class="kbd kbd-sm">"/"</kbd>
                </form>
                <a class="btn btn-ink btn-sm" href="/posts/new">"发布帖子"</a>
                <a class="btn btn-outline btn-sm" href="/admin">"管理后台"</a>
                <a class="btn btn-ghost btn-sm" href="/notifications">"通知 8"</a>
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
            <main class="mx-auto max-w-[1440px] px-6 py-8">
                {children()}
            </main>
        </div>
    }
}
