use serde::{Deserialize, Serialize};

use crate::domain::posts::PostSummary;

#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeQuery {
    pub tab: HomeTab,
    pub category: Option<String>,
    pub tag: Option<String>,
    pub time: HomeTimeRange,
    pub sort: HomeSort,
    pub page: usize,
    pub page_size: usize,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HomeTab {
    #[default]
    Latest,
    Hot,
    Unanswered,
    Following,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HomeTimeRange {
    #[default]
    All,
    Today,
    Week,
    Month,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HomeSort {
    #[default]
    LastReply,
    Created,
    Replies,
    Views,
    Hot,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TopicMarker {
    Pinned,
    Locked,
    Unread,
    Read,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomePageData {
    pub query: HomeQuery,
    pub topics: Vec<HomeTopic>,
    pub pagination: HomePagination,
    pub categories: Vec<HomeCategory>,
    pub hot_tags: Vec<HomeTag>,
    pub announcements: Vec<HomeAnnouncement>,
    pub active_authors: Vec<HomeActiveAuthor>,
    pub requires_login: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeTopic {
    pub id: String,
    pub marker: TopicMarker,
    pub title: String,
    pub summary: String,
    pub category: HomeCategoryBadge,
    pub tags: Vec<HomeTag>,
    pub reply_count: u32,
    pub view_count_label: String,
    pub last_reply: HomeLastReply,
    pub hot_score: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeCategoryBadge {
    pub name: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeTag {
    pub name: String,
    pub count: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeLastReply {
    pub author: String,
    pub avatar_label: String,
    pub time_label: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomePagination {
    pub page: usize,
    pub page_size: usize,
    pub total: usize,
    pub total_pages: usize,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeCategory {
    pub name: String,
    pub count: u32,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeAnnouncement {
    pub title: String,
    pub date_label: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct HomeActiveAuthor {
    pub name: String,
    pub avatar_label: String,
    pub reply_count_label: String,
}

impl HomeQuery {
    pub fn normalized(mut self) -> Self {
        if self.page == 0 {
            self.page = 1;
        }
        if self.page_size == 0 {
            self.page_size = 12;
        }
        self.page_size = self.page_size.min(50);
        self.category = normalize_filter(self.category);
        self.tag = normalize_filter(self.tag).map(|tag| tag.to_lowercase());
        self
    }
}

pub fn dense_workbench_home(query: HomeQuery, logged_in: bool) -> HomePageData {
    let query = query.normalized();
    let requires_login = query.tab == HomeTab::Following && !logged_in;
    let mut topics = if requires_login {
        Vec::new()
    } else {
        filter_topics(dense_workbench_topics(), &query)
    };

    sort_topics(&mut topics, query.sort, query.tab);

    HomePageData {
        query: query.clone(),
        topics,
        pagination: HomePagination {
            page: query.page,
            page_size: query.page_size,
            total: 342,
            total_pages: 29,
            label: format!("显示 1-{} / 342 个主题", query.page_size),
        },
        categories: seed_categories(),
        hot_tags: seed_hot_tags(),
        announcements: seed_announcements(),
        active_authors: seed_active_authors(),
        requires_login,
    }
}

pub fn home_topic_from_post_summary(summary: PostSummary) -> HomeTopic {
    let category_name = summary.category_name.unwrap_or_else(|| "讨论".to_string());
    let author_name = summary.author_name;

    HomeTopic {
        id: summary.post_id.to_string(),
        marker: TopicMarker::Unread,
        title: summary.title,
        summary: summary.summary,
        category: HomeCategoryBadge {
            color: category_color(&category_name).to_string(),
            name: category_name,
        },
        tags: summary
            .tags
            .into_iter()
            .map(|name| HomeTag { name, count: 0 })
            .collect(),
        reply_count: summary.comment_count.max(0) as u32,
        view_count_label: compact_count(summary.view_count),
        last_reply: HomeLastReply {
            avatar_label: author_name.chars().next().unwrap_or('P').to_string(),
            author: author_name,
            time_label: if summary.published_at.is_some() {
                "已发布".to_string()
            } else {
                "草稿".to_string()
            },
        },
        hot_score: summary.view_count
            + summary.comment_count * 20
            + summary.like_count * 10
            + summary.favorite_count * 5,
    }
}

fn compact_count(count: i64) -> String {
    if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.max(0).to_string()
    }
}

fn category_color(category: &str) -> &'static str {
    match category {
        "公告" => "blue",
        "教程" => "green",
        "问题" => "orange",
        "经验分享" => "sky",
        "站务" => "purple",
        _ => "gray",
    }
}

fn normalize_filter(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty() && value != "all")
}

fn filter_topics(topics: Vec<HomeTopic>, query: &HomeQuery) -> Vec<HomeTopic> {
    topics
        .into_iter()
        .filter(|topic| match query.tab {
            HomeTab::Unanswered => topic.reply_count == 0,
            _ => true,
        })
        .filter(|topic| {
            query
                .category
                .as_ref()
                .is_none_or(|category| topic.category.name == *category)
        })
        .filter(|topic| {
            query
                .tag
                .as_ref()
                .is_none_or(|tag| topic.tags.iter().any(|topic_tag| topic_tag.name == *tag))
        })
        .take(query.page_size)
        .collect()
}

fn sort_topics(topics: &mut [HomeTopic], sort: HomeSort, tab: HomeTab) {
    match (tab, sort) {
        (HomeTab::Hot, _) | (_, HomeSort::Hot) => {
            topics.sort_by(|left, right| right.hot_score.cmp(&left.hot_score));
        }
        (_, HomeSort::Replies) => {
            topics.sort_by(|left, right| right.reply_count.cmp(&left.reply_count));
        }
        (_, HomeSort::Views) => {
            topics.sort_by(|left, right| right.view_count_label.cmp(&left.view_count_label));
        }
        _ => {}
    }
}

pub fn dense_workbench_topics() -> Vec<HomeTopic> {
    vec![
        topic(
            "leptos-release",
            TopicMarker::Pinned,
            "Leptos 0.6 发布：更快的编译、更小的体积、Signal 优化",
            "包含编译性能改进、Signal 内存优化、SSR 稳定性提升和迁移注意事项。",
            "公告",
            "blue",
            &["leptos"],
            12,
            "3.2k",
            "张晨",
            "2 小时前",
            980,
        ),
        topic(
            "fullstack-guide",
            TopicMarker::Pinned,
            "新手指南：从 Axum + Leptos + SQLx 搭建全栈应用",
            "一步步搭建一个完整 CRUD 应用，包含认证、数据库操作和 SSR 渲染。",
            "教程",
            "green",
            &["axum", "sqlx", "+1"],
            28,
            "7.8k",
            "李明",
            "5 小时前",
            1200,
        ),
        topic(
            "rules",
            TopicMarker::Locked,
            "站点规则与发帖规范（必读）",
            "请在发帖前阅读本站规则，帮助我们保持高质量的技术讨论环境。",
            "站务",
            "purple",
            &["规则"],
            3,
            "9.1k",
            "管理员",
            "3 天前",
            860,
        ),
        topic(
            "server-function-sqlx",
            TopicMarker::Unread,
            "在 server function 中使用 SQLx 事务的最佳实践",
            "如何在 Leptos server function 中正确管理 SQLx 事务边界，避免连接泄漏。",
            "问题",
            "orange",
            &["leptos", "sqlx", "+1"],
            7,
            "452",
            "wangxy",
            "1 小时前",
            700,
        ),
        topic(
            "flyio-deploy",
            TopicMarker::Unread,
            "Leptos + Axum 部署到 Fly.io 的完整流程",
            "分享 Leptos SSR 应用部署到 Fly.io 的配置、构建和环境变量设置。",
            "经验分享",
            "sky",
            &["leptos", "axum", "部署"],
            5,
            "613",
            "DreamMao",
            "3 小时前",
            690,
        ),
        topic(
            "markdown-highlight",
            TopicMarker::Unread,
            "Markdown 渲染时如何高亮显示 Rust 代码？",
            "在 Leptos 中集成 pulldown-cmark 和 syntect，实现代码块高亮。",
            "问题",
            "orange",
            &["markdown", "rust", "+1"],
            3,
            "289",
            "coderLin",
            "昨天 22:15",
            520,
        ),
        topic(
            "wasm-size",
            TopicMarker::Unread,
            "Leptos WebAssembly 包大小优化实践",
            "通过裁剪特性、增加缓存和 wasm-opt 减少包体积。",
            "经验分享",
            "sky",
            &["wasm", "leptos", "优化"],
            9,
            "1.1k",
            "Skyline",
            "昨天 18:42",
            760,
        ),
        topic(
            "resources-repeat",
            TopicMarker::Read,
            "关于 resources! 宏在条件渲染下重复请求的问题",
            "当资源依赖发生变化且组件被重新挂载时，会触发重复请求，如何避免？",
            "问题",
            "orange",
            &["leptos", "resources"],
            2,
            "163",
            "小林",
            "昨天 11:03",
            300,
        ),
        topic(
            "jsonb-config",
            TopicMarker::Read,
            "使用 PostgreSQL JSONB 存储配置的方案选择",
            "在配置灵活性和查询性能之间如何权衡？求推荐实践。",
            "讨论",
            "gray",
            &["postgresql", "jsonb"],
            6,
            "342",
            "不二",
            "2 天前",
            460,
        ),
        topic(
            "signals-performance",
            TopicMarker::Read,
            "Leptos Signals 与派生状态的性能陷阱",
            "在大型列表和复杂计算中，如何避免不必要的派生和内存分配。",
            "讨论",
            "gray",
            &["leptos", "signals"],
            4,
            "276",
            "ChenKai",
            "2 天前",
            430,
        ),
        topic(
            "axum-body",
            TopicMarker::Read,
            "Axum 中间件处理 request body 的正确方式",
            "如何在不消耗 body 的情况下读取并复用请求体。",
            "问题",
            "orange",
            &["axum", "middleware"],
            1,
            "198",
            "ZhangT",
            "2 天前",
            280,
        ),
        topic(
            "component-library",
            TopicMarker::Read,
            "从零实现一个简单的 Leptos 组件库",
            "记录组件库从脚手架到发布 crates.io 的全过程。",
            "经验分享",
            "sky",
            &["leptos", "组件库"],
            0,
            "512",
            "Evan",
            "3 天前",
            500,
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn topic(
    id: &str,
    marker: TopicMarker,
    title: &str,
    summary: &str,
    category: &str,
    color: &str,
    tags: &[&str],
    reply_count: u32,
    view_count_label: &str,
    author: &str,
    time_label: &str,
    hot_score: i64,
) -> HomeTopic {
    HomeTopic {
        id: id.to_string(),
        marker,
        title: title.to_string(),
        summary: summary.to_string(),
        category: HomeCategoryBadge {
            name: category.to_string(),
            color: color.to_string(),
        },
        tags: tags
            .iter()
            .map(|name| HomeTag {
                name: (*name).to_string(),
                count: 0,
            })
            .collect(),
        reply_count,
        view_count_label: view_count_label.to_string(),
        last_reply: HomeLastReply {
            author: author.to_string(),
            avatar_label: author.chars().next().unwrap_or('P').to_string(),
            time_label: time_label.to_string(),
        },
        hot_score,
    }
}

fn seed_categories() -> Vec<HomeCategory> {
    vec![
        category("公告", 12, "blue"),
        category("教程", 34, "green"),
        category("问题", 156, "orange"),
        category("经验分享", 78, "sky"),
        category("讨论", 45, "gray"),
        category("站务", 17, "purple"),
    ]
}

fn category(name: &str, count: u32, color: &str) -> HomeCategory {
    HomeCategory {
        name: name.to_string(),
        count,
        color: color.to_string(),
    }
}

fn seed_hot_tags() -> Vec<HomeTag> {
    [
        ("leptos", 132),
        ("axum", 98),
        ("sqlx", 86),
        ("postgresql", 64),
        ("rust", 61),
        ("wasm", 48),
        ("server-functions", 42),
        ("markdown", 38),
    ]
    .into_iter()
    .map(|(name, count)| HomeTag {
        name: name.to_string(),
        count,
    })
    .collect()
}

fn seed_announcements() -> Vec<HomeAnnouncement> {
    [
        ("Leptos 0.6 正式发布", "5 月 20 日"),
        ("论坛升级与搜索增强说明", "5 月 10 日"),
        ("标签体系调整公告", "4 月 28 日"),
    ]
    .into_iter()
    .map(|(title, date_label)| HomeAnnouncement {
        title: title.to_string(),
        date_label: date_label.to_string(),
    })
    .collect()
}

fn seed_active_authors() -> Vec<HomeActiveAuthor> {
    [
        ("张晨", "1.2k 条回复"),
        ("DreamMao", "980 条回复"),
        ("Skyline", "876 条回复"),
        ("wangxy", "745 条回复"),
        ("coderLin", "612 条回复"),
    ]
    .into_iter()
    .map(|(name, reply_count_label)| HomeActiveAuthor {
        name: name.to_string(),
        avatar_label: name.chars().next().unwrap_or('P').to_string(),
        reply_count_label: reply_count_label.to_string(),
    })
    .collect()
}
