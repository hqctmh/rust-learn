use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::home::{
        HomeActiveAuthor, HomeAnnouncement, HomeCategory, HomeQuery, HomeTag, HomeTopic,
        TopicMarker, dense_workbench_home,
    },
};

#[component]
pub fn HomePage() -> impl IntoView {
    let home = dense_workbench_home(HomeQuery::default(), false);
    let topics = home.topics.clone();
    let categories = home.categories.clone();
    let hot_tags = home.hot_tags.clone();
    let announcements = home.announcements.clone();
    let active_authors = home.active_authors.clone();
    let pagination_label = home.pagination.label.clone();

    view! {
        <PageShell>
            <div class="workbench-layout">
                <section class="min-w-0">
                    <WorkbenchFilters/>
                    <div class="topic-table">
                        <div class="topic-head">
                            <span>"主题"</span>
                            <span>"分类"</span>
                            <span>"标签"</span>
                            <span class="text-center">"回复"</span>
                            <span class="text-center">"查看"</span>
                            <span>"最后回复"</span>
                        </div>
                        <For
                            each=move || topics.clone()
                            key=|topic| topic.id.clone()
                            children=move |topic| view! { <TopicRow topic/> }
                        />
                        <div class="pager-row">
                            <span>{pagination_label}</span>
                            <nav class="pager" aria-label="分页">
                                <a class="pager-button muted" href="/">"‹"</a>
                                <a class="pager-button active" href="/">"1"</a>
                                <a class="pager-button" href="/">"2"</a>
                                <a class="pager-button" href="/">"3"</a>
                                <span>"..."</span>
                                <a class="pager-button" href="/">"29"</a>
                                <a class="pager-button muted" href="/">"›"</a>
                            </nav>
                        </div>
                    </div>
                </section>
                <aside class="side-stack">
                    <CategoryPanel categories/>
                    <TagPanel tags=hot_tags/>
                    <AnnouncementPanel announcements/>
                    <AuthorPanel authors=active_authors/>
                </aside>
            </div>
        </PageShell>
    }
}

#[component]
fn WorkbenchFilters() -> impl IntoView {
    view! {
        <div class="workbench-toolbar">
            <div class="segmented-tabs">
                <a class="segment active" href="/">"最新"</a>
                <a class="segment" href="/">"热门"</a>
                <a class="segment" href="/">"未回复"</a>
                <a class="segment" href="/">"关注"</a>
            </div>
            <div class="filter-group">
                <button class="filter-pill">"所有分类"</button>
                <button class="filter-pill">"所有标签"</button>
                <button class="filter-pill">"所有时间"</button>
                <button class="filter-pill">"热度排序"</button>
            </div>
        </div>
    }
}

#[component]
fn TopicRow(topic: HomeTopic) -> impl IntoView {
    let marker_class = match topic.marker {
        TopicMarker::Pinned => "pin",
        TopicMarker::Locked => "lock",
        TopicMarker::Unread => "dot",
        TopicMarker::Read => "muted",
    };
    let tags = topic.tags.clone();

    view! {
        <a class="topic-row" href=format!("/posts/{}", topic.id)>
            <div class=format!("topic-marker {}", marker_class) aria-hidden="true"></div>
            <div class="topic-main">
                <h2>{topic.title}</h2>
                <p>{topic.summary}</p>
            </div>
            <div><span class=format!("badge badge-{}", topic.category.color)>{topic.category.name}</span></div>
            <div class="tag-list">
                {tags.into_iter().map(|tag| view! {
                    <span class="badge badge-soft">{tag.name}</span>
                }).collect_view()}
            </div>
            <div class="metric-cell">{topic.reply_count}</div>
            <div class="metric-cell">{topic.view_count_label}</div>
            <div class="last-reply">
                <span class="avatar-mini">{topic.last_reply.avatar_label}</span>
                <span><strong>{topic.last_reply.author}</strong><small>{topic.last_reply.time_label}</small></span>
            </div>
        </a>
    }
}

#[component]
fn CategoryPanel(categories: Vec<HomeCategory>) -> impl IntoView {
    view! {
        <section class="panel-card">
            <h2>"分类"</h2>
            <ul class="category-list">
                {categories.into_iter().map(|category| view! {
                    <li>
                        <span><i class=format!("category-dot {}", category.color)></i>{category.name}</span>
                        <strong>{category.count}</strong>
                    </li>
                }).collect_view()}
            </ul>
            <a class="link-action" href="/">"查看全部分类 →"</a>
        </section>
    }
}

#[component]
fn TagPanel(tags: Vec<HomeTag>) -> impl IntoView {
    view! {
        <section class="panel-card">
            <h2>"热门标签"</h2>
            <div class="tag-grid">
                {tags.into_iter().map(|tag| view! {
                    <a class="tag-chip" href="/"><span>{tag.name}</span><small>{tag.count}</small></a>
                }).collect_view()}
            </div>
            <a class="link-action" href="/">"查看全部标签 →"</a>
        </section>
    }
}

#[component]
fn AnnouncementPanel(announcements: Vec<HomeAnnouncement>) -> impl IntoView {
    view! {
        <section class="panel-card">
            <h2>"公告"</h2>
            <ul class="compact-list">
                {announcements.into_iter().map(|announcement| view! {
                    <li><span>{announcement.title}</span><time>{announcement.date_label}</time></li>
                }).collect_view()}
            </ul>
            <a class="link-action" href="/">"查看全部公告 →"</a>
        </section>
    }
}

#[component]
fn AuthorPanel(authors: Vec<HomeActiveAuthor>) -> impl IntoView {
    view! {
        <section class="panel-card">
            <h2>"活跃作者"</h2>
            <ul class="author-list">
                {authors.into_iter().map(|author| view! {
                    <li>
                        <span class="avatar-mini">{author.avatar_label}</span>
                        <span>{author.name}</span>
                        <small>{author.reply_count_label}</small>
                    </li>
                }).collect_view()}
            </ul>
            <a class="link-action" href="/">"查看全部作者 →"</a>
        </section>
    }
}
