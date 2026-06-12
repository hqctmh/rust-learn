use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::home::{HomeActiveAuthor, HomeAnnouncement, HomeCategory, HomeTag, HomeTopic},
    page_data::fallback_home_page,
};

#[component]
pub fn PostsIndexPage() -> impl IntoView {
    let home = fallback_home_page();
    let topics = home.topics;

    view! {
        <PageShell>
            <PublicIndexShell title="帖子" subtitle="推荐帖子列表，浏览最新和热门主题流">
                <div class="public-index-list">
                    {topics.into_iter().map(|topic| view! { <PostIndexItem topic/> }).collect_view()}
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn CategoriesIndexPage() -> impl IntoView {
    let home = fallback_home_page();
    let categories = home.categories;

    view! {
        <PageShell>
            <PublicIndexShell title="分类" subtitle="按讨论主题进入首页筛选视图">
                <div class="public-index-grid">
                    {categories.into_iter().map(|category| view! { <CategoryIndexItem category/> }).collect_view()}
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn TagsIndexPage() -> impl IntoView {
    let home = fallback_home_page();
    let tags = home.hot_tags;

    view! {
        <PageShell>
            <PublicIndexShell title="标签" subtitle="查看设计稿中的热门标签和使用量">
                <div class="public-index-grid compact">
                    {tags.into_iter().map(|tag| view! { <TagIndexItem tag/> }).collect_view()}
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn AnnouncementsIndexPage() -> impl IntoView {
    let home = fallback_home_page();
    let announcements = home.announcements;

    view! {
        <PageShell>
            <PublicIndexShell title="公告" subtitle="查看论坛公开公告和运营通知">
                <div class="public-index-list">
                    {announcements.into_iter().map(|announcement| view! { <AnnouncementIndexItem announcement/> }).collect_view()}
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn UsersIndexPage() -> impl IntoView {
    let home = fallback_home_page();
    let authors = home.active_authors;

    view! {
        <PageShell>
            <PublicIndexShell title="活跃作者" subtitle="按回复活跃度查看公开作者榜单">
                <div class="public-index-list">
                    {authors.into_iter().map(|author| view! { <AuthorIndexItem author/> }).collect_view()}
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn DocsIndexPage() -> impl IntoView {
    view! {
        <PageShell>
            <PublicIndexShell title="文档" subtitle="沉淀论坛使用说明和技术写作规范">
                <div class="public-index-grid">
                    <DocIndexItem title="Markdown 发帖指南" summary="标题、摘要、标签、代码块和图片上传的推荐写法。" href="/posts/new"/>
                    <DocIndexItem title="SQLx 宏查询规范" summary="运行态 SQL 查询需要使用 checked macro，并保持 schema 可验证。" href="/search?q=sqlx"/>
                    <DocIndexItem title="社区内容规范" summary="发帖、回复、举报和公告阅读的公开规则入口。" href="/search?q=规则"/>
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
pub fn ActivitiesIndexPage() -> impl IntoView {
    view! {
        <PageShell>
            <PublicIndexShell title="活动" subtitle="查看社区近期技术活动和协作主题">
                <div class="public-index-list">
                    <ActivityIndexItem title="线上分享会：Leptos SSR 实战" date="6 月 20 日" href="/search?q=leptos"/>
                    <ActivityIndexItem title="SQLx 查询优化共创" date="6 月 27 日" href="/search?q=sqlx"/>
                    <ActivityIndexItem title="Rust 全栈项目回顾" date="7 月 4 日" href="/search?q=rust"/>
                </div>
            </PublicIndexShell>
        </PageShell>
    }
}

#[component]
fn PublicIndexShell(
    title: &'static str,
    subtitle: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="public-index-page">
            <header class="public-index-header">
                <div>
                    <div class="page-kicker">"Post Forum"</div>
                    <h1>{title}</h1>
                    <p>{subtitle}</p>
                </div>
                <a class="btn btn-outline btn-sm" href="/">"返回首页"</a>
            </header>
            {children()}
        </div>
    }
}

#[component]
fn PostIndexItem(topic: HomeTopic) -> impl IntoView {
    let href = format!("/posts/{}", topic.id);
    let tags = topic.tags.clone();

    view! {
        <a class="public-index-item topic" href=href>
            <span>{topic.title}</span>
            <strong>{topic.view_count_label}</strong>
            <p>{topic.summary}</p>
            <small>
                {topic.category.name}
                " · "
                {topic.reply_count}
                " 条回复"
            </small>
            <em>"查看帖子详情"</em>
            <div class="tag-list">
                {tags.into_iter().map(|tag| view! { <b class="badge badge-soft">{tag.name}</b> }).collect_view()}
            </div>
        </a>
    }
}

#[component]
fn CategoryIndexItem(category: HomeCategory) -> impl IntoView {
    let href = format!("/?tab=latest&category={}", category.name);

    view! {
        <a class="public-index-item" href=href>
            <span><i class=format!("category-dot {}", category.color)></i>{category.name}</span>
            <strong>{category.count}</strong>
            <em>"查看相关主题"</em>
        </a>
    }
}

#[component]
fn TagIndexItem(tag: HomeTag) -> impl IntoView {
    let href = format!("/search?tag={}", tag.name);

    view! {
        <a class="public-index-item" href=href>
            <span>{format!("#{}", tag.name)}</span>
            <strong>{tag.count}</strong>
            <em>"查看相关主题"</em>
        </a>
    }
}

#[component]
fn DocIndexItem(title: &'static str, summary: &'static str, href: &'static str) -> impl IntoView {
    view! {
        <a class="public-index-item topic" href=href>
            <span>{title}</span>
            <strong>"文档"</strong>
            <p>{summary}</p>
            <em>"查看文档相关内容"</em>
        </a>
    }
}

#[component]
fn ActivityIndexItem(title: &'static str, date: &'static str, href: &'static str) -> impl IntoView {
    view! {
        <a class="public-index-item" href=href>
            <span>{title}</span>
            <strong>{date}</strong>
            <em>"查看活动相关主题"</em>
        </a>
    }
}

#[component]
fn AnnouncementIndexItem(announcement: HomeAnnouncement) -> impl IntoView {
    view! {
        <a class="public-index-item" href="/announcements">
            <span>{announcement.title}</span>
            <strong>{announcement.date_label}</strong>
            <em>"查看公告详情"</em>
        </a>
    }
}

#[component]
fn AuthorIndexItem(author: HomeActiveAuthor) -> impl IntoView {
    let href = format!("/search?q={}", author.name);

    view! {
        <a class="public-index-item author" href=href>
            <span><b class="avatar-mini">{author.avatar_label}</b>{author.name}</span>
            <strong>{author.reply_count_label}</strong>
            <em>"查看相关主题"</em>
        </a>
    }
}
