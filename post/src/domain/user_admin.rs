use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminUserRow {
    pub user_id: Uuid,
    pub username: String,
    pub nickname: String,
    pub roles: Vec<String>,
    pub disabled: bool,
    pub post_count: usize,
    pub comment_count: usize,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuditContext {
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateUserRolesRequest {
    pub roles: Vec<String>,
    pub context: AuditContext,
}

impl UpdateUserRolesRequest {
    pub fn normalized_roles(&self) -> Vec<String> {
        self.roles
            .iter()
            .map(|role| role.trim().to_lowercase())
            .filter(|role| !role.is_empty())
            .fold(Vec::new(), |mut acc, role| {
                if !acc.contains(&role) {
                    acc.push(role);
                }
                acc
            })
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuditLogEntry {
    pub audit_id: Uuid,
    pub actor_id: Uuid,
    pub actor_name: String,
    pub action: String,
    pub target_type: String,
    pub target_id: Uuid,
    pub target_label: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: OffsetDateTime,
}
