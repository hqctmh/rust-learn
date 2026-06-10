use serde::{Deserialize, Serialize};

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

pub fn admin_permissions() -> Vec<Permission> {
    [
        ("user:view", "查看用户"),
        ("post:view", "查看帖子"),
        ("post:update", "更新帖子"),
        ("comment:view", "查看评论"),
        ("comment:delete", "删除评论"),
        ("announcement:publish", "发布公告"),
        ("audit:view", "查看审计日志"),
    ]
    .into_iter()
    .map(|(code, name)| Permission::new(code, name))
    .collect()
}
