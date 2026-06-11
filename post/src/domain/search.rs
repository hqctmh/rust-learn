use serde::{Deserialize, Serialize};

use crate::domain::home::{HomeTopic, dense_workbench_topics};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchQuery {
    pub q: String,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub sort: SearchSort,
    pub page: usize,
    pub page_size: usize,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            q: String::new(),
            category: None,
            tag: None,
            sort: SearchSort::Relevance,
            page: 1,
            page_size: 10,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchSort {
    #[default]
    Relevance,
    Latest,
    Hot,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultKind {
    Post,
    Tag,
    User,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchResultPage {
    pub query: SearchQuery,
    pub total: usize,
    pub items: Vec<SearchResultItem>,
    pub suggestions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchResultItem {
    pub id: String,
    pub kind: SearchResultKind,
    pub title: String,
    pub title_highlighted: String,
    pub summary: String,
    pub summary_highlighted: String,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub author_name: String,
    pub score: i64,
    pub url: String,
}

impl SearchQuery {
    pub fn normalized(mut self) -> Self {
        self.q = self.q.trim().to_string();
        self.category = normalize_filter(self.category);
        self.tag = normalize_filter(self.tag).map(|tag| tag.to_lowercase());
        if self.page == 0 {
            self.page = 1;
        }
        if self.page_size == 0 {
            self.page_size = 10;
        }
        self.page_size = self.page_size.min(50);
        self
    }
}

pub fn search_dense_workbench(query: SearchQuery) -> SearchResultPage {
    let query = query.normalized();
    let needle = query.q.to_lowercase();
    let mut items = dense_workbench_topics()
        .into_iter()
        .filter(|topic| matches_query(topic, &needle))
        .filter(|topic| {
            query
                .category
                .as_ref()
                .is_none_or(|category| topic.category.name == *category)
        })
        .filter(|topic| {
            query
                .tag
                .as_ref()
                .is_none_or(|tag| topic.tags.iter().any(|topic_tag| topic_tag.name == *tag))
        })
        .map(|topic| to_result_item(topic, &query.q))
        .collect::<Vec<_>>();

    sort_results(&mut items, query.sort);

    let total = items.len();
    let start = (query.page - 1) * query.page_size;
    let items = items
        .into_iter()
        .skip(start)
        .take(query.page_size)
        .collect();

    SearchResultPage {
        query,
        total,
        items,
        suggestions: vec![
            "leptos server functions".to_string(),
            "sqlx transaction".to_string(),
            "axum middleware".to_string(),
        ],
    }
}

fn normalize_filter(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "all")
}

fn matches_query(topic: &HomeTopic, needle: &str) -> bool {
    if needle.is_empty() {
        return true;
    }

    let haystack = format!(
        "{} {} {} {} {}",
        topic.title,
        topic.summary,
        topic.category.name,
        topic
            .tags
            .iter()
            .map(|tag| tag.name.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        topic.last_reply.author
    )
    .to_lowercase();

    haystack.contains(needle)
}

fn to_result_item(topic: HomeTopic, query: &str) -> SearchResultItem {
    SearchResultItem {
        id: topic.id.clone(),
        kind: SearchResultKind::Post,
        title_highlighted: highlight(&topic.title, query),
        summary_highlighted: highlight(&topic.summary, query),
        title: topic.title,
        summary: topic.summary,
        category_name: Some(topic.category.name),
        tags: topic.tags.into_iter().map(|tag| tag.name).collect(),
        author_name: topic.last_reply.author,
        score: topic.hot_score,
        url: format!("/posts/{}", topic.id),
    }
}

fn highlight(value: &str, query: &str) -> String {
    let query = query.trim();
    if query.is_empty() {
        return value.to_string();
    }

    let lower_value = value.to_lowercase();
    let lower_query = query.to_lowercase();
    if let Some(start) = lower_value.find(&lower_query) {
        let end = start + lower_query.len();
        format!(
            "{}<mark>{}</mark>{}",
            &value[..start],
            &value[start..end],
            &value[end..]
        )
    } else {
        value.to_string()
    }
}

fn sort_results(items: &mut [SearchResultItem], sort: SearchSort) {
    match sort {
        SearchSort::Hot => items.sort_by(|left, right| right.score.cmp(&left.score)),
        SearchSort::Latest | SearchSort::Relevance => {}
    }
}
