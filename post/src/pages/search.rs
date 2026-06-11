use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::search::{SearchQuery, search_dense_workbench},
};

#[component]
pub fn SearchPage() -> impl IntoView {
    let results = search_dense_workbench(SearchQuery {
        q: "sqlx".to_string(),
        ..Default::default()
    });
    let items = results.items.clone();
    let suggestions = results.suggestions.clone();

    view! {
        <PageShell>
            <div class="search-page">
                <section class="search-header-panel">
                    <div>
                        <div class="page-kicker">"全文搜索"</div>
                        <h1>"搜索帖子、标签、用户"</h1>
                        <p>"当前展示关键词 sqlx 的搜索结果，支持后续接入 Elasticsearch 替换本地索引。"</p>
                    </div>
                    <form class="search-page-form" action="/search" method="get">
                        <input name="q" type="search" value=results.query.q.clone() placeholder="搜索帖子、标签、用户..."/>
                        <button class="btn btn-ink" type="submit">"搜索"</button>
                    </form>
                </section>

                <div class="search-layout">
                    <aside class="search-filter-panel">
                        <h2>"筛选"</h2>
                        <a class="filter-chip active" href="/search?q=sqlx">"全部结果"</a>
                        <a class="filter-chip" href="/search?q=sqlx&category=问题">"问题"</a>
                        <a class="filter-chip" href="/search?q=sqlx&tag=leptos">"leptos"</a>
                        <a class="filter-chip" href="/search?q=sqlx&sort=hot">"热度排序"</a>
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
        </PageShell>
    }
}
