use serde::{Deserialize, Serialize};

use crate::domain::model::User;

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

#[derive(Debug,Serialize)]
pub struct LoginResponse {
    pub user: User,
    pub access_token: String,
    pub token_type: &'static str,
    pub expires_in: u64,
}
