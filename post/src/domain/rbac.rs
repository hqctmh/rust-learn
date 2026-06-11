use serde::{Deserialize, Serialize};

use crate::domain::user_admin::AuditContext;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Permission {
    pub code: String,
    pub name: String,
}

impl Permission {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct Role {
    pub code: String,
    pub name: String,
    pub permissions: Vec<Permission>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct CreateRoleRequest {
    pub code: String,
    pub name: String,
    pub permission_codes: Vec<String>,
    pub context: AuditContext,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct UpdateRoleRequest {
    pub name: Option<String>,
    pub permission_codes: Option<Vec<String>>,
    pub context: AuditContext,
}

pub fn admin_permissions() -> Vec<Permission> {
    [
        ("user:view", "查看用户"),
        ("user:update", "更新用户"),
        ("user:disable", "禁用用户"),
        ("post:view", "查看帖子"),
        ("post:update", "更新帖子"),
        ("post:delete", "删除帖子"),
        ("comment:view", "查看评论"),
        ("comment:delete", "删除评论"),
        ("announcement:create", "创建公告"),
        ("announcement:publish", "发布公告"),
        ("role:view", "查看角色"),
        ("role:create", "创建角色"),
        ("role:update", "更新角色"),
        ("role:delete", "删除角色"),
        ("permission:view", "查看权限"),
        ("category:view", "查看分类"),
        ("tag:view", "查看标签"),
        ("report:view", "查看举报"),
        ("audit:view", "查看审计日志"),
    ]
    .into_iter()
    .map(|(code, name)| Permission::new(code, name))
    .collect()
}
