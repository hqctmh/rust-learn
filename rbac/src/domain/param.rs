use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UserPageQuery {
    pub username: Option<String>,
    pub display_name: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}
