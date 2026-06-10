use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct SessionUser {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
    pub is_admin: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub nickname: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Session {
    pub session_id: Uuid,
    pub user: SessionUser,
    pub expires_at: OffsetDateTime,
}
