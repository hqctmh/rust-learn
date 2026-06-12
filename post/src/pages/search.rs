use leptos::prelude::*;
use leptos_router::{hooks::use_query_map, params::ParamsMap};

use crate::{
    components::PageShell,
    domain::search::{SearchQuery, SearchResultPage, SearchSort},
    page_data::{fallback_search_page, load_search_page},
};

#[component]
pub fn SearchPage() -> impl IntoView {
    let query_map = use_query_map();
    let fallback_query_map = query_map;
    let suspense_query_map = query_map;
    let results = Resource::new(
        move || search_query_from_params(&query_map.read()),
        load_search_page,
    );

    view! {
        <PageShell>
            <Suspense fallback=move || view! {
                <SearchResults results=fallback_search_page(search_query_from_params(&fallback_query_map.read()))/>
            }>
                {move || {
                    let query = search_query_from_params(&suspense_query_map.read());
                    Suspend::new(async move {
                    let results = results.await.unwrap_or_else(|_| fallback_search_page(query.clone()));
                    view! { <SearchResults results/> }
                })}}
            </Suspense>
        </PageShell>
    }
}

fn search_query_from_params(params: &ParamsMap) -> SearchQuery {
    SearchQuery {
        q: params.get("q").unwrap_or_default(),
        category: params.get("category"),
        tag: params.get("tag"),
        sort: search_sort_from_param(params.get("sort").as_deref()),
        page: params
            .get("page")
            .and_then(|page| page.parse::<usize>().ok())
            .unwrap_or(1),
        page_size: params
            .get("page_size")
            .and_then(|page_size| page_size.parse::<usize>().ok())
            .unwrap_or(10),
    }
    .normalized()
}

fn search_sort_from_param(sort: Option<&str>) -> SearchSort {
    match sort {
        Some("latest") => SearchSort::Latest,
        Some("hot") => SearchSort::Hot,
        _ => SearchSort::Relevance,
    }
}

#[component]
fn SearchResults(results: SearchResultPage) -> impl IntoView {
    let items = results.items.clone();
    let suggestions = results.suggestions.clone();
    let all_href = search_href(&results.query, None, None, None);
    let issue_href = search_href(&results.query, Some("问题"), None, None);
    let leptos_href = search_href(&results.query, None, Some("leptos"), None);
    let hot_href = search_href(&results.query, None, None, Some(SearchSort::Hot));
    let current_keyword = if results.query.q.is_empty() {
        "全部内容".to_string()
    } else {
        results.query.q.clone()
    };

    view! {
        <div class="search-page">
            <section class="search-header-panel">
                <div>
                    <div class="page-kicker">"全文搜索"</div>
                    <h1>"搜索帖子、标签、用户"</h1>
                    <p>"当前展示关键词 "{current_keyword}" 的搜索结果，支持后续接入 Elasticsearch 替换本地索引。"</p>
                </div>
                <form class="search-page-form" action="/search" method="get">
                    <input name="q" type="search" value=results.query.q.clone() placeholder="搜索帖子、标签、用户..."/>
                    <button class="btn btn-ink" type="submit">"搜索"</button>
                </form>
            </section>

            <div class="search-layout">
                <aside class="search-filter-panel">
                    <h2>"筛选"</h2>
                    <a class="filter-chip active" href=all_href>"全部结果"</a>
                    <a class="filter-chip" href=issue_href>"问题"</a>
                    <a class="filter-chip" href=leptos_href>"leptos"</a>
                    <a class="filter-chip" href=hot_href>"热度排序"</a>
                    <div class="suggestion-list">
                        <span>"搜索建议"</span>
                        {suggestions.into_iter().map(|suggestion| {
                            let href = format!("/search?q={suggestion}");
                            view! { <a href=href>{suggestion}</a> }
                        }).collect_view()}
                    </div>
                </aside>

                <section class="search-results-panel">
                    <div class="result-summary">
                        <strong>{results.total}</strong>
                        <span>" 个结果匹配 “"{results.query.q.clone()}"”"</span>
                    </div>
                    <div class="search-result-list">
                        {items.into_iter().map(|item| view! {
                            <a class="search-result-row" href=item.url>
                                <div class="result-kind">"帖子"</div>
                                <h2 inner_html=item.title_highlighted></h2>
                                <p inner_html=item.summary_highlighted></p>
                                <div class="result-meta">
                                    <span>{item.category_name.unwrap_or_else(|| "未分类".to_string())}</span>
                                    <span>{item.author_name}</span>
                                    {item.tags.into_iter().map(|tag| view! { <em>{tag}</em> }).collect_view()}
                                </div>
                            </a>
                        }).collect_view()}
                    </div>
                </section>
            </div>
        </div>
    }
}

fn search_href(
    query: &SearchQuery,
    category: Option<&str>,
    tag: Option<&str>,
    sort: Option<SearchSort>,
) -> String {
    let mut href = format!("/search?q={}", query.q);
    if let Some(category) = category.or(query.category.as_deref()) {
        href.push_str("&category=");
        href.push_str(category);
    }
    if let Some(tag) = tag.or(query.tag.as_deref()) {
        href.push_str("&tag=");
        href.push_str(tag);
    }
    let sort = sort.unwrap_or(query.sort);
    match sort {
        SearchSort::Relevance => {}
        SearchSort::Latest => href.push_str("&sort=latest"),
        SearchSort::Hot => href.push_str("&sort=hot"),
    }
    href
}
