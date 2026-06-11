use time::OffsetDateTime;
use uuid::Uuid;

use crate::{
    domain::{
        auth::SessionUser,
        user_admin::{AdminUserRow, AuditContext, AuditLogEntry},
    },
    error::ForumError,
};

pub struct UserAdminService;

impl UserAdminService {
    pub fn ensure_not_self_disable(actor_id: Uuid, target_id: Uuid) -> Result<(), ForumError> {
        if actor_id == target_id {
            return Err(ForumError::Conflict("不能禁用自己".to_string()));
        }

        Ok(())
    }

    pub fn normalize_roles(roles: Vec<String>) -> Result<Vec<String>, ForumError> {
        let roles = roles
            .iter()
            .map(|role| role.trim().to_lowercase())
            .filter(|role| !role.is_empty())
            .fold(Vec::new(), |mut acc, role| {
                if !acc.contains(&role) {
                    acc.push(role);
                }
                acc
            });

        if roles.is_empty() {
            return Err(ForumError::Validation("角色不能为空".to_string()));
        }

        Ok(roles)
    }

    pub fn admin_user_row(
        user: &SessionUser,
        roles: Vec<String>,
        disabled: bool,
        post_count: usize,
        comment_count: usize,
    ) -> AdminUserRow {
        AdminUserRow {
            user_id: user.user_id,
            username: user.username.clone(),
            nickname: user.nickname.clone(),
            roles,
            disabled,
            post_count,
            comment_count,
        }
    }

    pub fn audit_snapshot(row: &AdminUserRow) -> String {
        format!(
            "username={},disabled={},roles={}",
            row.username,
            row.disabled,
            row.roles.join("|")
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn build_audit_log(
        audit_id: Uuid,
        actor: &SessionUser,
        action: &str,
        target_type: &str,
        target_id: Uuid,
        target_label: String,
        before: Option<String>,
        after: Option<String>,
        context: AuditContext,
        created_at: OffsetDateTime,
    ) -> AuditLogEntry {
        AuditLogEntry {
            audit_id,
            actor_id: actor.user_id,
            actor_name: actor.nickname.clone(),
            action: action.to_string(),
            target_type: target_type.to_string(),
            target_id,
            target_label,
            before,
            after,
            ip: context.ip,
            user_agent: context.user_agent,
            created_at,
        }
    }
}
