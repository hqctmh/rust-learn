use leptos::prelude::*;
use leptos_router::{hooks::use_query_map, params::ParamsMap};

use crate::{
    components::PageShell,
    domain::home::{
        HomeActiveAuthor, HomeAnnouncement, HomeCategory, HomePageData, HomeQuery, HomeSort,
        HomeTab, HomeTag, HomeTimeRange, HomeTopic, TopicMarker,
    },
    page_data::{fallback_home_page, load_home_page},
};

#[component]
pub fn HomePage() -> impl IntoView {
    let query_map = use_query_map();
    let fallback_query_map = query_map;
    let suspense_query_map = query_map;
    let home = Resource::new(
        move || home_query_from_params(&query_map.read()),
        load_home_page,
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <HomeWorkbench home=home_fallback_for_query(home_query_from_params(&fallback_query_map.read()))/>
            }>
                {move || {
                    let query = home_query_from_params(&suspense_query_map.read());
                    Suspend::new(async move {
                    let home = home.await.unwrap_or_else(|_| home_fallback_for_query(query.clone()));
                    view! { <HomeWorkbench home/> }
                })}}
            </Suspense>
        </PageShell>
    }
}

fn home_query_from_params(params: &ParamsMap) -> HomeQuery {
    HomeQuery {
        tab: home_tab_from_param(params.get("tab").as_deref()),
        category: params.get("category"),
        tag: params.get("tag"),
        time: home_time_from_param(params.get("time").as_deref()),
        sort: home_sort_from_param(params.get("sort").as_deref()),
        page: params
            .get("page")
            .and_then(|page| page.parse::<usize>().ok())
            .unwrap_or(1),
        page_size: params
            .get("page_size")
            .and_then(|page_size| page_size.parse::<usize>().ok())
            .unwrap_or(12),
    }
    .normalized()
}

fn home_fallback_for_query(query: HomeQuery) -> HomePageData {
    let mut home = fallback_home_page();
    home.query = query;
    home
}

#[component]
fn HomeWorkbench(home: HomePageData) -> impl IntoView {
    let query = home.query.clone();
    let topics = home.topics.clone();
    let categories = home.categories.clone();
    let hot_tags = home.hot_tags.clone();
    let announcements = home.announcements.clone();
    let active_authors = home.active_authors.clone();
    let pagination = home.pagination.clone();
    let pagination_label = home.pagination.label.clone();
    let page_first = home_href(&query, None, None, None, None, None, Some(1));
    let page_second = home_href(&query, None, None, None, None, None, Some(2));
    let page_third = home_href(&query, None, None, None, None, None, Some(3));
    let page_last = home_href(
        &query,
        None,
        None,
        None,
        None,
        None,
        Some(pagination.total_pages.max(1)),
    );
    let prev_page = pagination.page.saturating_sub(1).max(1);
    let next_page = (pagination.page + 1).min(pagination.total_pages.max(1));
    let page_prev = home_href(&query, None, None, None, None, None, Some(prev_page));
    let page_next = home_href(&query, None, None, None, None, None, Some(next_page));

    view! {
        <div class="workbench-layout">
            <section class="min-w-0">
                <WorkbenchFilters query=query.clone()/>
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
                            <a class="pager-button muted" href=page_prev>"‹"</a>
                            <a class="pager-button active" href=page_first>"1"</a>
                            <a class="pager-button" href=page_second>"2"</a>
                            <a class="pager-button" href=page_third>"3"</a>
                            <span>"..."</span>
                            <a class="pager-button" href=page_last>{pagination.total_pages}</a>
                            <a class="pager-button muted" href=page_next>"›"</a>
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
    }
}

#[component]
fn WorkbenchFilters(query: HomeQuery) -> impl IntoView {
    let latest_href = home_href(
        &query,
        Some(HomeTab::Latest),
        None,
        None,
        None,
        None,
        Some(1),
    );
    let hot_href = home_href(&query, Some(HomeTab::Hot), None, None, None, None, Some(1));
    let unanswered_href = home_href(
        &query,
        Some(HomeTab::Unanswered),
        None,
        None,
        None,
        None,
        Some(1),
    );
    let following_href = home_href(
        &query,
        Some(HomeTab::Following),
        None,
        None,
        None,
        None,
        Some(1),
    );
    let all_categories_href = home_href(&query, None, Some(None), None, None, None, Some(1));
    let all_tags_href = home_href(&query, None, None, Some(None), None, None, Some(1));
    let all_time_href = home_href(
        &query,
        None,
        None,
        None,
        Some(HomeTimeRange::All),
        None,
        Some(1),
    );
    let hot_sort_href = home_href(&query, None, None, None, None, Some(HomeSort::Hot), Some(1));

    view! {
        <div class="workbench-toolbar">
            <div class="segmented-tabs">
                <a class=if query.tab == HomeTab::Latest { "segment active" } else { "segment" } href=latest_href>"最新"</a>
                <a class=if query.tab == HomeTab::Hot { "segment active" } else { "segment" } href=hot_href>"热门"</a>
                <a class=if query.tab == HomeTab::Unanswered { "segment active" } else { "segment" } href=unanswered_href>"未回复"</a>
                <a class=if query.tab == HomeTab::Following { "segment active" } else { "segment" } href=following_href>"关注"</a>
            </div>
            <div class="filter-group">
                <a class="filter-pill" href=all_categories_href>"所有分类"</a>
                <a class="filter-pill" href=all_tags_href>"所有标签"</a>
                <a class="filter-pill" href=all_time_href>"所有时间"</a>
                <a class="filter-pill" href=hot_sort_href>"热度排序"</a>
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
            <a class="link-action" href="/?tab=latest&category=all">"查看全部分类 →"</a>
        </section>
    }
}

#[component]
fn TagPanel(tags: Vec<HomeTag>) -> impl IntoView {
    view! {
        <section class="panel-card">
            <h2>"热门标签"</h2>
            <div class="tag-grid">
                {tags.into_iter().map(|tag| {
                    let href = format!("/?tab=latest&tag={}", tag.name);
                    view! {
                    <a class="tag-chip" href=href><span>{tag.name}</span><small>{tag.count}</small></a>
                    }
                }).collect_view()}
            </div>
            <a class="link-action" href="/?tab=latest&tag=all">"查看全部标签 →"</a>
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
            <a class="link-action" href="/?tab=latest&sort=created">"查看全部公告 →"</a>
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
            <a class="link-action" href="/?tab=hot&sort=replies">"查看全部作者 →"</a>
        </section>
    }
}

fn home_tab_from_param(tab: Option<&str>) -> HomeTab {
    match tab {
        Some("hot") => HomeTab::Hot,
        Some("unanswered") => HomeTab::Unanswered,
        Some("following") => HomeTab::Following,
        _ => HomeTab::Latest,
    }
}

fn home_time_from_param(time: Option<&str>) -> HomeTimeRange {
    match time {
        Some("today") => HomeTimeRange::Today,
        Some("week") => HomeTimeRange::Week,
        Some("month") => HomeTimeRange::Month,
        _ => HomeTimeRange::All,
    }
}

fn home_sort_from_param(sort: Option<&str>) -> HomeSort {
    match sort {
        Some("created") => HomeSort::Created,
        Some("replies") => HomeSort::Replies,
        Some("views") => HomeSort::Views,
        Some("hot") => HomeSort::Hot,
        _ => HomeSort::LastReply,
    }
}

fn home_href(
    query: &HomeQuery,
    tab: Option<HomeTab>,
    category: Option<Option<&str>>,
    tag: Option<Option<&str>>,
    time: Option<HomeTimeRange>,
    sort: Option<HomeSort>,
    page: Option<usize>,
) -> String {
    let mut next = query.clone();
    if let Some(tab) = tab {
        next.tab = tab;
    }
    if let Some(category) = category {
        next.category = category.map(ToString::to_string);
    }
    if let Some(tag) = tag {
        next.tag = tag.map(ToString::to_string);
    }
    if let Some(time) = time {
        next.time = time;
    }
    if let Some(sort) = sort {
        next.sort = sort;
    }
    if let Some(page) = page {
        next.page = page;
    }
    next = next.normalized();

    let mut href = format!("/?tab={}", home_tab_param(next.tab));
    if let Some(category) = next.category {
        href.push_str("&category=");
        href.push_str(&category);
    }
    if let Some(tag) = next.tag {
        href.push_str("&tag=");
        href.push_str(&tag);
    }
    if next.time != HomeTimeRange::All {
        href.push_str("&time=");
        href.push_str(home_time_param(next.time));
    }
    if next.sort != HomeSort::LastReply {
        href.push_str("&sort=");
        href.push_str(home_sort_param(next.sort));
    }
    if next.page > 1 {
        href.push_str("&page=");
        href.push_str(&next.page.to_string());
    }
    href
}

fn home_tab_param(tab: HomeTab) -> &'static str {
    match tab {
        HomeTab::Latest => "latest",
        HomeTab::Hot => "hot",
        HomeTab::Unanswered => "unanswered",
        HomeTab::Following => "following",
    }
}

fn home_time_param(time: HomeTimeRange) -> &'static str {
    match time {
        HomeTimeRange::All => "all",
        HomeTimeRange::Today => "today",
        HomeTimeRange::Week => "week",
        HomeTimeRange::Month => "month",
    }
}

fn home_sort_param(sort: HomeSort) -> &'static str {
    match sort {
        HomeSort::LastReply => "last_reply",
        HomeSort::Created => "created",
        HomeSort::Replies => "replies",
        HomeSort::Views => "views",
        HomeSort::Hot => "hot",
    }
}
