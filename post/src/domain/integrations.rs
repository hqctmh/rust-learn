use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::{
    announcements::AnnouncementItem,
    posts::{PostDetail, PostStatus},
};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum IntegrationAction {
    CacheInvalidate(CacheInvalidation),
    NatsPublish(IntegrationEvent),
    SearchIndex(SearchIndexMutation),
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CacheInvalidation {
    pub keys: Vec<String>,
    pub reason: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct IntegrationEvent {
    pub subject: String,
    pub aggregate_id: Uuid,
    pub payload_summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum SearchIndexMutation {
    Upsert(SearchIndexDocument),
    Delete { index: String, document_id: Uuid },
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SearchIndexDocument {
    pub index: String,
    pub document_id: Uuid,
    pub title: String,
    pub summary: String,
    pub body: String,
    pub category_name: Option<String>,
    pub tags: Vec<String>,
    pub author_id: Uuid,
}

pub fn post_published_actions(post: &PostDetail) -> Vec<IntegrationAction> {
    vec![
        IntegrationAction::CacheInvalidate(CacheInvalidation {
            keys: vec![
                "home:topics:*".to_string(),
                "home:sidebar:*".to_string(),
                "posts:hot".to_string(),
                "tags:hot".to_string(),
                "authors:active".to_string(),
            ],
            reason: "post.published".to_string(),
        }),
        IntegrationAction::NatsPublish(IntegrationEvent {
            subject: "post.published".to_string(),
            aggregate_id: post.summary.post_id,
            payload_summary: post.summary.title.clone(),
        }),
        IntegrationAction::SearchIndex(SearchIndexMutation::Upsert(post_search_document(post))),
    ]
}

pub fn post_comment_changed_actions(post: &PostDetail, comment_id: Uuid) -> Vec<IntegrationAction> {
    vec![
        IntegrationAction::CacheInvalidate(CacheInvalidation {
            keys: vec![
                format!("post:{}:comments", post.summary.post_id),
                format!("post:{}:detail", post.summary.post_id),
                "home:topics:*".to_string(),
                "authors:active".to_string(),
            ],
            reason: "comment.changed".to_string(),
        }),
        IntegrationAction::NatsPublish(IntegrationEvent {
            subject: "comment.created".to_string(),
            aggregate_id: comment_id,
            payload_summary: post.summary.title.clone(),
        }),
        IntegrationAction::SearchIndex(SearchIndexMutation::Upsert(post_search_document(post))),
    ]
}

pub fn announcement_published_actions(announcement: &AnnouncementItem) -> Vec<IntegrationAction> {
    vec![
        IntegrationAction::CacheInvalidate(CacheInvalidation {
            keys: vec![
                "home:announcements".to_string(),
                "home:sidebar:*".to_string(),
            ],
            reason: "announcement.published".to_string(),
        }),
        IntegrationAction::NatsPublish(IntegrationEvent {
            subject: "announcement.published".to_string(),
            aggregate_id: announcement.announcement_id,
            payload_summary: announcement.title.clone(),
        }),
    ]
}

fn post_search_document(post: &PostDetail) -> SearchIndexDocument {
    let index = if post.status == PostStatus::Published {
        "posts".to_string()
    } else {
        "posts-drafts".to_string()
    };

    SearchIndexDocument {
        index,
        document_id: post.summary.post_id,
        title: post.summary.title.clone(),
        summary: post.summary.summary.clone(),
        body: post.markdown.clone(),
        category_name: post.summary.category_name.clone(),
        tags: post.summary.tags.clone(),
        author_id: post.summary.author_id,
    }
}
