use std::collections::HashMap;

use crate::{
    domain::rbac::{Permission, Role, UpdateRoleRequest, admin_permissions},
    error::ForumError,
};

pub struct RbacService;

impl RbacService {
    pub fn normalize_role_code(value: &str) -> Result<String, ForumError> {
        let code = value.trim().to_lowercase();
        if code.is_empty() {
            return Err(ForumError::Validation("角色编码不能为空".to_string()));
        }
        if code.chars().count() > 32
            || !code
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-' || ch == '_')
        {
            return Err(ForumError::Validation(
                "角色编码只能包含小写字母、数字、- 和 _，且不能超过 32 个字符".to_string(),
            ));
        }
        Ok(code)
    }

    pub fn normalize_role_name(value: &str) -> Result<String, ForumError> {
        let name = value.trim();
        if name.is_empty() {
            return Err(ForumError::Validation("角色名称不能为空".to_string()));
        }
        if name.chars().count() > 32 {
            return Err(ForumError::Validation(
                "角色名称不能超过 32 个字符".to_string(),
            ));
        }
        Ok(name.to_string())
    }

    pub fn resolve_permissions(codes: &[String]) -> Result<Vec<Permission>, ForumError> {
        let available = admin_permissions()
            .into_iter()
            .map(|permission| (permission.code.clone(), permission))
            .collect::<HashMap<_, _>>();
        let mut permissions = Vec::new();
        for code in codes {
            let code = code.trim();
            if code.is_empty() {
                continue;
            }
            let permission = available
                .get(code)
                .cloned()
                .ok_or_else(|| ForumError::Validation(format!("权限不存在: {code}")))?;
            if !permissions
                .iter()
                .any(|item: &Permission| item.code == permission.code)
            {
                permissions.push(permission);
            }
        }
        if permissions.is_empty() {
            return Err(ForumError::Validation("角色权限不能为空".to_string()));
        }
        Ok(permissions)
    }

    pub fn build_role(
        code: &str,
        name: &str,
        permission_codes: &[String],
    ) -> Result<Role, ForumError> {
        Ok(Role {
            code: Self::normalize_role_code(code)?,
            name: Self::normalize_role_name(name)?,
            permissions: Self::resolve_permissions(permission_codes)?,
        })
    }

    pub fn apply_role_update(
        role: &mut Role,
        request: UpdateRoleRequest,
    ) -> Result<(), ForumError> {
        if let Some(name) = request.name {
            role.name = Self::normalize_role_name(&name)?;
        }
        if let Some(permission_codes) = request.permission_codes {
            role.permissions = Self::resolve_permissions(&permission_codes)?;
        }
        Ok(())
    }

    pub fn ensure_deletable_role(code: &str) -> Result<(), ForumError> {
        let code = Self::normalize_role_code(code)?;
        if code == "admin" || code == "member" {
            return Err(ForumError::Conflict("内置角色不能删除".to_string()));
        }
        Ok(())
    }
}
