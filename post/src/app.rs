use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::pages::{
    admin::AdminPage, editor::EditorPage, home::HomePage, login::LoginPage,
    post_detail::PostDetailPage,
};

pub fn primary_routes() -> Vec<&'static str> {
    vec!["/", "/posts/new", "/login", "/admin"]
}

pub fn home_seed_text() -> &'static str {
    "推荐 最新 热门 发布帖子 评论 管理端 分类 标签 公告 活跃作者"
}

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="zh-CN">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" href="/favicon.svg" type="image/svg+xml"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/post.css"/>
        <Title text="Post Forum"/>
        <Router>
            <Routes fallback=|| view! { <main class="min-h-screen bg-base-200 p-8">"页面不存在"</main> }>
                <Route path=path!("") view=HomePage/>
                <Route path=path!("posts/new") view=EditorPage/>
                <Route path=path!("posts/:id") view=PostDetailPage/>
                <Route path=path!("login") view=LoginPage/>
                <Route path=path!("admin") view=AdminPage/>
            </Routes>
        </Router>
    }
}
