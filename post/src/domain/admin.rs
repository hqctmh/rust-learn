use serde::{Deserialize, Serialize};

use crate::domain::rbac::{Permission, admin_permissions};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminDashboard {
    pub stats: Vec<AdminStat>,
    pub menu: Vec<AdminMenuItem>,
    pub permissions: Vec<Permission>,
    pub users: Vec<AdminUserRow>,
    pub moderation_posts: Vec<AdminPostRow>,
    pub moderation_comments: Vec<AdminCommentRow>,
    pub categories: Vec<AdminCategoryRow>,
    pub tags: Vec<AdminTagRow>,
    pub announcements: Vec<AdminAnnouncementRow>,
    pub reports: Vec<AdminReportRow>,
    pub governance_queue: Vec<GovernanceQueueItem>,
    pub audit_entries: Vec<AuditEntry>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminStat {
    pub label: String,
    pub value: String,
    pub delta: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminMenuItem {
    pub label: String,
    pub permission: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminUserRow {
    pub username: String,
    pub nickname: String,
    pub roles: Vec<String>,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminPostRow {
    pub title: String,
    pub author: String,
    pub category: String,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminCommentRow {
    pub post_title: String,
    pub author: String,
    pub content: String,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminReportRow {
    pub target: String,
    pub target_type: String,
    pub reason: String,
    pub reporter: String,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminCategoryRow {
    pub name: String,
    pub color: String,
    pub sort_order: i32,
    pub post_count: u32,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminTagRow {
    pub name: String,
    pub sort_order: i32,
    pub use_count: u32,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AdminAnnouncementRow {
    pub title: String,
    pub announcement_type: String,
    pub audience: String,
    pub status: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct GovernanceQueueItem {
    pub label: String,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuditEntry {
    pub actor: String,
    pub action: String,
    pub target: String,
    pub ip: String,
    pub user_agent: String,
    pub time_label: String,
}

pub fn admin_dashboard_demo() -> AdminDashboard {
    AdminDashboard {
        stats: vec![
            stat("用户总数", "12,480", "今日 +38"),
            stat("帖子总数", "3,426", "今日 +64"),
            stat("评论总数", "18,902", "今日 +210"),
            stat("在线用户", "328", "WebSocket"),
        ],
        menu: vec![
            menu("系统统计", "audit:view"),
            menu("用户管理", "user:view"),
            menu("角色管理", "role:view"),
            menu("权限管理", "permission:view"),
            menu("帖子管理", "post:view"),
            menu("评论管理", "comment:view"),
            menu("分类管理", "category:view"),
            menu("标签管理", "tag:view"),
            menu("公告推送", "announcement:publish"),
            menu("举报处理", "report:view"),
            menu("审计日志", "audit:view"),
        ],
        permissions: admin_permissions(),
        users: vec![
            AdminUserRow {
                username: "mah".to_string(),
                nickname: "mah".to_string(),
                roles: vec!["admin".to_string()],
                status: "正常".to_string(),
                actions: vec!["调整角色".to_string(), "禁用用户".to_string()],
            },
            AdminUserRow {
                username: "managed-user".to_string(),
                nickname: "managed-user".to_string(),
                roles: vec!["member".to_string()],
                status: "已禁用".to_string(),
                actions: vec!["解禁用户".to_string(), "调整角色".to_string()],
            },
        ],
        moderation_posts: vec![
            AdminPostRow {
                title: "Leptos + Axum 构建全栈应用".to_string(),
                author: "Skyline".to_string(),
                category: "经验分享".to_string(),
                status: "已发布".to_string(),
                actions: vec![
                    "下架".to_string(),
                    "置顶".to_string(),
                    "取消置顶".to_string(),
                    "删除".to_string(),
                ],
            },
            AdminPostRow {
                title: "表单验证实践".to_string(),
                author: "hello-rust".to_string(),
                category: "教程".to_string(),
                status: "已下架".to_string(),
                actions: vec!["恢复".to_string(), "查看".to_string(), "删除".to_string()],
            },
        ],
        moderation_comments: vec![
            AdminCommentRow {
                post_title: "在 server function 中使用 SQLx 事务的最佳实践".to_string(),
                author: "wangxy".to_string(),
                content: "这个事务边界解释得很清楚".to_string(),
                status: "正常".to_string(),
                actions: vec!["删除评论".to_string(), "查看帖子".to_string()],
            },
            AdminCommentRow {
                post_title: "Markdown 渲染时如何高亮显示 Rust 代码？".to_string(),
                author: "visitor".to_string(),
                content: "该评论已被删除".to_string(),
                status: "已删除".to_string(),
                actions: vec!["恢复评论".to_string(), "查看帖子".to_string()],
            },
        ],
        categories: vec![
            AdminCategoryRow {
                name: "公告".to_string(),
                color: "#0064E0".to_string(),
                sort_order: 1,
                post_count: 12,
                status: "启用".to_string(),
                actions: vec![
                    "编辑".to_string(),
                    "调整排序".to_string(),
                    "停用".to_string(),
                ],
            },
            AdminCategoryRow {
                name: "教程".to_string(),
                color: "#35A853".to_string(),
                sort_order: 2,
                post_count: 34,
                status: "启用".to_string(),
                actions: vec![
                    "编辑".to_string(),
                    "调整排序".to_string(),
                    "停用".to_string(),
                ],
            },
        ],
        tags: vec![
            AdminTagRow {
                name: "leptos".to_string(),
                sort_order: 1,
                use_count: 132,
                status: "启用".to_string(),
                actions: vec![
                    "编辑".to_string(),
                    "合并标签".to_string(),
                    "禁用".to_string(),
                ],
            },
            AdminTagRow {
                name: "axum".to_string(),
                sort_order: 2,
                use_count: 98,
                status: "启用".to_string(),
                actions: vec![
                    "编辑".to_string(),
                    "合并标签".to_string(),
                    "禁用".to_string(),
                ],
            },
        ],
        announcements: vec![
            AdminAnnouncementRow {
                title: "论坛升级与搜索增强说明".to_string(),
                announcement_type: "系统公告".to_string(),
                audience: "全体用户".to_string(),
                status: "已发布".to_string(),
                actions: vec!["下线公告".to_string(), "推送公告".to_string()],
            },
            AdminAnnouncementRow {
                title: "标签体系调整公告".to_string(),
                announcement_type: "运营公告".to_string(),
                audience: "指定角色".to_string(),
                status: "草稿".to_string(),
                actions: vec!["发布公告".to_string(), "编辑".to_string()],
            },
        ],
        reports: vec![
            AdminReportRow {
                target: "Markdown 渲染时如何高亮显示 Rust 代码？".to_string(),
                target_type: "帖子".to_string(),
                reason: "垃圾广告".to_string(),
                reporter: "wangxy".to_string(),
                status: "待处理".to_string(),
                actions: vec![
                    "标记已处理".to_string(),
                    "驳回".to_string(),
                    "删除违规内容".to_string(),
                ],
            },
            AdminReportRow {
                target: "关于 resources! 宏在条件渲染下重复请求的问题".to_string(),
                target_type: "评论".to_string(),
                reason: "人身攻击".to_string(),
                reporter: "Skyline".to_string(),
                status: "待处理".to_string(),
                actions: vec!["标记已处理".to_string(), "驳回".to_string()],
            },
        ],
        governance_queue: vec![
            queue("举报处理", "23 待处理"),
            queue("评论管理", "8 条被举报评论"),
            queue("分类管理", "6 个一级分类"),
            queue("标签管理", "128 个标签，可合并"),
            queue("公告推送", "NATS 全体 / 角色 / 指定用户"),
            queue("审计日志", "记录 IP / User-Agent"),
        ],
        audit_entries: vec![
            AuditEntry {
                actor: "管理员".to_string(),
                action: "置顶帖子".to_string(),
                target: "Leptos 0.6 发布".to_string(),
                ip: "127.0.0.1".to_string(),
                user_agent: "Post Forum Admin".to_string(),
                time_label: "10 分钟前".to_string(),
            },
            AuditEntry {
                actor: "运营".to_string(),
                action: "发布公告".to_string(),
                target: "论坛升级与搜索增强说明".to_string(),
                ip: "127.0.0.1".to_string(),
                user_agent: "Post Forum Admin".to_string(),
                time_label: "1 小时前".to_string(),
            },
        ],
    }
}

fn stat(label: &str, value: &str, delta: &str) -> AdminStat {
    AdminStat {
        label: label.to_string(),
        value: value.to_string(),
        delta: delta.to_string(),
    }
}

fn menu(label: &str, permission: &str) -> AdminMenuItem {
    AdminMenuItem {
        label: label.to_string(),
        permission: permission.to_string(),
    }
}

fn queue(label: &str, value: &str) -> GovernanceQueueItem {
    GovernanceQueueItem {
        label: label.to_string(),
        value: value.to_string(),
    }
}
