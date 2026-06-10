use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Category {
    pub category_id: Uuid,
    pub name: String,
    pub slug: String,
    pub sort_order: i32,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Tag {
    pub tag_id: Uuid,
    pub name: String,
    pub slug: String,
    pub created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TaxonomyInput {
    pub name: String,
    pub slug: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct TagInput {
    pub name: String,
    pub slug: String,
}
