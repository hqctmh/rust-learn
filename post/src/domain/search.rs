use serde::{Deserialize, Serialize};

use crate::domain::home::{
    HomeActiveAuthor, HomeQuery, HomeTag, HomeTopic, dense_workbench_home, dense_workbench_topics,
};

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

    if query.category.is_none() && query.tag.is_none() {
        let home = dense_workbench_home(HomeQuery::default(), true);
        items.extend(
            home.hot_tags
                .into_iter()
                .filter(|tag| matches_tag_query(tag, &needle))
                .map(|tag| tag_to_result_item(tag, &query.q)),
        );
        items.extend(
            home.active_authors
                .into_iter()
                .filter(|author| matches_author_query(author, &needle))
                .map(|author| author_to_result_item(author, &query.q)),
        );
    }

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

fn matches_tag_query(tag: &HomeTag, needle: &str) -> bool {
    needle.is_empty() || tag.name.to_lowercase().contains(needle)
}

fn matches_author_query(author: &HomeActiveAuthor, needle: &str) -> bool {
    needle.is_empty() || author.name.to_lowercase().contains(needle)
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

fn tag_to_result_item(tag: HomeTag, query: &str) -> SearchResultItem {
    let title = format!("#{}", tag.name);
    SearchResultItem {
        id: tag.name.clone(),
        kind: SearchResultKind::Tag,
        title_highlighted: highlight(&title, query),
        summary_highlighted: highlight(&format!("{} 个主题使用该标签", tag.count), query),
        title,
        summary: format!("{} 个主题使用该标签", tag.count),
        category_name: None,
        tags: vec![tag.name.clone()],
        author_name: "标签".to_string(),
        score: i64::from(tag.count),
        url: format!("/search?tag={}", tag.name),
    }
}

fn author_to_result_item(author: HomeActiveAuthor, query: &str) -> SearchResultItem {
    let user_id = user_search_id(&author.name);
    let summary = format!("活跃作者，{}", author.reply_count_label);
    let score = author_score(&author.reply_count_label);
    SearchResultItem {
        id: user_id.clone(),
        kind: SearchResultKind::User,
        title_highlighted: highlight(&author.name, query),
        summary_highlighted: highlight(&summary, query),
        title: author.name,
        summary,
        category_name: None,
        tags: Vec::new(),
        author_name: "用户".to_string(),
        score,
        url: format!("/users/{user_id}"),
    }
}

fn user_search_id(name: &str) -> String {
    let ascii = name
        .chars()
        .filter(|value| value.is_ascii_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    if !ascii.is_empty() {
        return ascii;
    }

    let encoded = name
        .chars()
        .map(|value| format!("{:x}", value as u32))
        .collect::<Vec<_>>()
        .join("-");
    format!("u-{encoded}")
}

fn author_score(reply_count_label: &str) -> i64 {
    let number = reply_count_label
        .split_whitespace()
        .next()
        .unwrap_or_default()
        .trim_end_matches('k')
        .parse::<f64>()
        .unwrap_or_default();
    if reply_count_label.contains('k') {
        (number * 1000.0) as i64
    } else {
        number as i64
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
