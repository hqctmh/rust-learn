use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::pages::{
    admin::AdminPage,
    editor::EditorPage,
    home::HomePage,
    login::LoginPage,
    notifications::NotificationsPage,
    post_detail::PostDetailPage,
    register::RegisterPage,
    search::SearchPage,
    user_space::{MePage, UserProfilePage},
};

pub fn primary_routes() -> Vec<&'static str> {
    vec![
        "/",
        "/search",
        "/notifications",
        "/users/sample",
        "/me",
        "/me/posts",
        "/me/drafts",
        "/me/comments",
        "/me/favorites",
        "/me/following",
        "/me/followers",
        "/posts/new",
        "/posts/sample/edit",
        "/login",
        "/register",
        "/admin",
    ]
}

pub fn api_route_inventory() -> Vec<&'static str> {
    vec![
        "/api/home",
        "/api/categories",
        "/api/tags",
        "/api/announcements",
        "/api/announcements/{announcement_id}/read",
        "/api/admin/dashboard",
        "/api/admin/categories",
        "/api/admin/categories/{category_id}/update",
        "/api/admin/categories/{category_id}/disable",
        "/api/admin/tags",
        "/api/admin/tags/{tag_id}/update",
        "/api/admin/tags/{tag_id}/merge",
        "/api/admin/tags/{tag_id}/delete",
        "/api/admin/posts",
        "/api/admin/posts/{post_id}/offline",
        "/api/admin/posts/{post_id}/restore",
        "/api/admin/posts/{post_id}/delete",
        "/api/admin/posts/{post_id}/pin",
        "/api/admin/posts/{post_id}/unpin",
        "/api/admin/comments",
        "/api/admin/comments/{comment_id}/delete",
        "/api/admin/comments/{comment_id}/recover",
        "/api/admin/users",
        "/api/admin/users/{user_id}/disable",
        "/api/admin/users/{user_id}/enable",
        "/api/admin/users/{user_id}/roles",
        "/api/admin/roles",
        "/api/admin/roles/{role_code}/update",
        "/api/admin/roles/{role_code}/delete",
        "/api/admin/permissions",
        "/api/admin/audit-logs",
        "/api/admin/announcements",
        "/api/admin/announcements/{announcement_id}/publish",
        "/api/admin/announcements/{announcement_id}/withdraw",
        "/api/admin/reports",
        "/api/admin/reports/{report_id}/handle",
        "/api/files",
        "/api/files/binary",
        "/api/login",
        "/api/register",
        "/api/logout",
        "/api/session/{session_id}",
        "/api/notifications",
        "/ws/notifications/{user_id}",
        "/api/notifications/online",
        "/api/notifications/pending-pushes",
        "/api/notifications/pending-pushes/{push_id}/ack",
        "/api/notifications/{notification_id}/read",
        "/api/notifications/read-all",
        "/api/posts/drafts/autosave",
        "/api/posts",
        "/api/search",
        "/api/posts/{post_id}",
        "/api/posts/{post_id}/update",
        "/api/posts/{post_id}/delete",
        "/api/posts/{post_id}/comments",
        "/api/comments/{comment_id}/delete",
        "/api/comments/{comment_id}/like",
        "/api/comments/{comment_id}/report",
        "/api/posts/{post_id}/like",
        "/api/posts/{post_id}/favorite",
        "/api/reports",
        "/api/users/{user_id}/profile",
        "/api/users/{user_id}/avatar",
        "/api/users/{user_id}/password",
        "/api/users/{user_id}/follow",
        "/api/users/{user_id}/space",
    ]
}

pub fn home_seed_text() -> &'static str {
    "Post Forum 首页 帖子 标签 用户 文档 活动 搜索帖子、标签、用户... 发布帖子 管理后台 管理端 通知 登录 推荐 最新 热门 未回复 关注 所有分类 所有标签 所有时间 主题 分类 标签 回复 查看 最后回复 显示 1-12 / 342 个主题 热门标签 公告 活跃作者 评论"
}

pub fn ui_feature_inventory() -> &'static str {
    "关注动态 分类过滤 标签过滤 分页 Markdown 编辑 图片上传 MIME 类型 文件大小 Markdown 图片链接 实时预览 代码高亮 自动保存 点赞 收藏 关注作者 举报 回复评论 相关推荐 WebSocket 推送 消息中心 全部已读 全文搜索 搜索高亮 个人主页 草稿 RustFS NATS Elasticsearch RBAC 用户管理 角色管理 权限管理 帖子管理 评论管理 分类管理 标签管理 公告推送 举报处理 审计日志 系统统计"
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
                <Route path=path!("search") view=SearchPage/>
                <Route path=path!("notifications") view=NotificationsPage/>
                <Route path=path!("users/:id") view=UserProfilePage/>
                <Route path=path!("me") view=MePage/>
                <Route path=path!("me/posts") view=MePage/>
                <Route path=path!("me/drafts") view=MePage/>
                <Route path=path!("me/comments") view=MePage/>
                <Route path=path!("me/favorites") view=MePage/>
                <Route path=path!("me/following") view=MePage/>
                <Route path=path!("me/followers") view=MePage/>
                <Route path=path!("posts/new") view=EditorPage/>
                <Route path=path!("posts/:id/edit") view=EditorPage/>
                <Route path=path!("posts/:id") view=PostDetailPage/>
                <Route path=path!("login") view=LoginPage/>
                <Route path=path!("register") view=RegisterPage/>
                <Route path=path!("admin") view=AdminPage/>
            </Routes>
        </Router>
    }
}
