use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// 对应 `users` 表。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 对应 `roles` 表。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 对应 `permissions` 表。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// 对应 `user_roles` 用户角色关联表。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// 对应 `role_permissions` 角色权限关联表。
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
}
