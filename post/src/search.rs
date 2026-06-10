use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::domain::{
    events::ForumEvent,
    posts::{SearchQuery, SearchSort},
};

#[cfg(feature = "ssr")]
use crate::error::ForumError;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchPostDocument {
    pub post_id: Uuid,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub tags: Vec<String>,
    pub category_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchIndexOperation {
    Index { document: SearchPostDocument },
    Delete { post_id: Uuid },
}

impl SearchIndexOperation {
    pub fn from_event(event: &ForumEvent) -> Option<Self> {
        match event {
            ForumEvent::SearchPostIndex {
                post_id,
                title,
                body,
                tags,
            } => Some(Self::Index {
                document: SearchPostDocument {
                    post_id: *post_id,
                    title: title.clone(),
                    summary: title.clone(),
                    body: body.clone(),
                    tags: tags.clone(),
                    category_name: None,
                },
            }),
            ForumEvent::SearchPostDelete { post_id } => Some(Self::Delete { post_id: *post_id }),
            _ => None,
        }
    }
}

pub struct ElasticsearchPostIndexer {
    #[cfg(feature = "ssr")]
    index_name: String,
    #[cfg(feature = "ssr")]
    client: elasticsearch::Elasticsearch,
}

impl ElasticsearchPostIndexer {
    #[cfg(feature = "ssr")]
    pub fn new(endpoint: &str, index_name: impl Into<String>) -> Result<Self, ForumError> {
        let transport = elasticsearch::http::transport::Transport::single_node(endpoint)
            .map_err(|_| ForumError::Internal)?;
        Ok(Self {
            index_name: index_name.into(),
            client: elasticsearch::Elasticsearch::new(transport),
        })
    }

    pub fn search_body(query: &SearchQuery) -> Value {
        let keyword = query.keyword.as_deref().unwrap_or("*").trim();
        let mut must = Vec::new();
        if keyword.is_empty() || keyword == "*" {
            must.push(json!({ "match_all": {} }));
        } else {
            must.push(json!({
                "multi_match": {
                    "query": keyword,
                    "fields": ["title^3", "summary^2", "body", "tags", "category_name"]
                }
            }));
        }

        let mut filter = Vec::new();
        if let Some(category_name) = query
            .category_name
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
        {
            filter.push(json!({ "term": { "category_name.keyword": category_name } }));
        }
        if let Some(tag) = query
            .tag
            .as_ref()
            .map(|value| value.trim().trim_start_matches('#').to_lowercase())
            .filter(|value| !value.is_empty())
        {
            filter.push(json!({ "term": { "tags.keyword": tag } }));
        }

        let from = (query.page.max(1) - 1) * query.page_size.clamp(1, 100);
        let size = query.page_size.clamp(1, 100);
        let sort = match query.sort {
            SearchSort::Latest => json!([{ "published_at": { "order": "desc" } }]),
            SearchSort::Hot => json!([{ "score": { "order": "desc" } }]),
        };

        json!({
            "from": from,
            "size": size,
            "query": {
                "bool": {
                    "must": must,
                    "filter": filter
                }
            },
            "sort": sort,
            "highlight": {
                "fields": {
                    "title": {},
                    "summary": {},
                    "body": {}
                }
            }
        })
    }

    #[cfg(feature = "ssr")]
    pub async fn apply(&self, operation: &SearchIndexOperation) -> Result<(), ForumError> {
        use elasticsearch::{DeleteParts, IndexParts};

        match operation {
            SearchIndexOperation::Index { document } => {
                self.client
                    .index(IndexParts::IndexId(
                        &self.index_name,
                        &document.post_id.to_string(),
                    ))
                    .body(json!(document))
                    .send()
                    .await
                    .map_err(|_| ForumError::Internal)?;
            }
            SearchIndexOperation::Delete { post_id } => {
                self.client
                    .delete(DeleteParts::IndexId(&self.index_name, &post_id.to_string()))
                    .send()
                    .await
                    .map_err(|_| ForumError::Internal)?;
            }
        }
        Ok(())
    }

    #[cfg(feature = "ssr")]
    pub async fn search(&self, query: &SearchQuery) -> Result<Value, ForumError> {
        use elasticsearch::SearchParts;

        self.client
            .search(SearchParts::Index(&[&self.index_name]))
            .body(Self::search_body(query))
            .send()
            .await
            .map_err(|_| ForumError::Internal)?
            .json::<Value>()
            .await
            .map_err(|_| ForumError::Internal)
    }
}
