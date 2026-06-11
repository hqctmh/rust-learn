# Post Forum Phase 1 Homepage System Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the homepage system foundation required by `post/prd.md` and the provided Dense Workbench UI: 12-topic forum table, four-card sidebar, queryable homepage data, and a stable API/Server Function contract.

**Architecture:** Add a dedicated homepage domain module that owns the DTOs and query semantics, then have `ForumStore` produce `HomePageData` from seeded forum data. Expose the same data through `/api/home` first, then bind the Leptos homepage to the structured data so the UI no longer depends on ad-hoc static rows.

**Tech Stack:** Rust 2024, Leptos 0.8, Leptos Router 0.8, Axum 0.8, Serde, UUID, time, existing in-memory `ForumStore`, existing contract tests.

---

## Scope

This plan implements the first independently verifiable system slice. It does not complete every PRD subsystem in one pass; search indexing, Redis/NATS/RustFS production integration, full RBAC screens, WebSocket delivery, and PostgreSQL persistence will be handled by separate plans after this homepage foundation is stable.

The first slice must prove:

- Homepage visible content matches the design contract in `post/prd.md`.
- Frontend rows and sidebars are driven by typed homepage data.
- `/api/home` returns the same data contract needed by the design.
- Query semantics exist for `latest`, `hot`, `unanswered`, `following`, category, tag, time, sort, and pagination.
- Existing tests remain green.

## Current Evidence

- `post/src/pages/home.rs` currently hardcodes 10 rows, shows `显示 1-10 / 342 个主题`, and includes a `NotificationPanel` with “系统功能”.
- `post/prd.md` requires 12 rows, `显示 1-12 / 342 个主题`, and exactly four sidebar cards: 分类、热门标签、公告、活跃作者.
- `post/src/api.rs` currently exposes `/api/posts` but not `/api/home`.
- `post/src/domain/posts.rs` has `PostSummary`, but it lacks homepage-specific fields such as marker, category color, last reply, read state, and pagination metadata.

## File Structure

- Create: `post/src/domain/home.rs`
  - Owns homepage DTOs, query enums, pagination metadata, seeded design data, and deterministic filtering/sorting helpers.
- Modify: `post/src/domain/mod.rs`
  - Exposes the new `home` module.
- Modify: `post/src/state.rs`
  - Adds `ForumStore::home_page(HomeQuery, Option<Uuid>) -> Result<HomePageData, ForumError>`.
- Modify: `post/src/api.rs`
  - Adds `GET /api/home` using Axum `Query<HomeQueryParams>` and returns `Json<HomePageData>`.
- Modify: `post/src/pages/home.rs`
  - Replaces scattered static arrays with typed homepage data rendering.
  - Removes the forbidden “系统功能” sidebar card.
- Modify: `post/src/app.rs`
  - Updates the homepage seed contract text so tests cover the new design-specific requirements.
- Modify: `post/tests/phase1_contract.rs`
  - Adds contract tests for homepage data, query behavior, API route inventory, and rendered UI inventory.

## Context7 Findings Used

- Leptos 0.8 supports `#[server]` functions that return `Result<T, ServerFnError>` and can be called through `Action`/`Resource`; implementation can add Server Functions after the stable domain/API contract exists.
- Leptos 0.8 routing examples use `Router`, `Routes`, `Route`, `path!`, `use_query_map`, and `Suspense`/`Suspend` for query-driven data loading.
- Axum 0.8 uses `Query<T>` for query string extraction, `Json<T>` for JSON responses, and handler signatures can combine `Extension<ForumStore>` with `Query<HomeQueryParams>`.
- Axum extractor rejection handling can be customized, but this phase keeps query parsing resilient by using optional string fields and normalizing invalid values to defaults inside the domain layer.

## Tasks

### Task 1: Add Homepage Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Write failing tests for the homepage design contract**

Add these tests near the existing homepage tests:

```rust
#[test]
fn home_page_data_matches_dense_workbench_design_contract() {
    let store = post::state::ForumStore::seeded();
    let home = store
        .home_page(post::domain::home::HomeQuery::default(), None)
        .expect("home page data");

    assert_eq!(home.topics.len(), 12);
    assert_eq!(home.pagination.page, 1);
    assert_eq!(home.pagination.page_size, 12);
    assert_eq!(home.pagination.total, 342);
    assert_eq!(home.pagination.total_pages, 29);
    assert_eq!(home.pagination.label, "显示 1-12 / 342 个主题");

    let titles: Vec<_> = home.topics.iter().map(|topic| topic.title.as_str()).collect();
    for required in [
        "Leptos 0.6 发布：更快的编译、更小的体积、Signal 优化",
        "新手指南：从 Axum + Leptos + SQLx 搭建全栈应用",
        "站点规则与发帖规范（必读）",
        "从零实现一个简单的 Leptos 组件库",
    ] {
        assert!(titles.contains(&required), "missing topic {required}");
    }

    assert_eq!(home.categories.len(), 6);
    assert_eq!(home.hot_tags.len(), 8);
    assert_eq!(home.announcements.len(), 3);
    assert_eq!(home.active_authors.len(), 5);
}
```

- [ ] **Step 2: Write failing tests for query behavior**

Add this test:

```rust
#[test]
fn home_page_query_supports_tabs_filters_and_pagination_defaults() {
    let store = post::state::ForumStore::seeded();

    let unanswered = store
        .home_page(
            post::domain::home::HomeQuery {
                tab: post::domain::home::HomeTab::Unanswered,
                ..Default::default()
            },
            None,
        )
        .expect("unanswered home page");
    assert!(unanswered.topics.iter().all(|topic| topic.reply_count == 0));

    let leptos = store
        .home_page(
            post::domain::home::HomeQuery {
                tag: Some("leptos".to_string()),
                ..Default::default()
            },
            None,
        )
        .expect("tag filtered home page");
    assert!(
        leptos
            .topics
            .iter()
            .all(|topic| topic.tags.iter().any(|tag| tag.name == "leptos"))
    );

    let following = store
        .home_page(
            post::domain::home::HomeQuery {
                tab: post::domain::home::HomeTab::Following,
                ..Default::default()
            },
            None,
        )
        .expect("anonymous following home page");
    assert!(following.requires_login);
    assert!(following.topics.is_empty());
}
```

- [ ] **Step 3: Write failing tests for UI inventory and API route inventory**

Add this test:

```rust
#[test]
fn home_page_ui_inventory_matches_sidebar_and_pagination_contract() {
    let text = post::app::home_seed_text();

    for required in [
        "首页",
        "帖子",
        "标签",
        "用户",
        "文档",
        "活动",
        "搜索帖子、标签、用户...",
        "显示 1-12 / 342 个主题",
        "分类",
        "热门标签",
        "公告",
        "活跃作者",
    ] {
        assert!(text.contains(required), "missing {required}");
    }

    assert!(!text.contains("系统功能"));
}

#[test]
fn api_routes_include_homepage_aggregate_endpoint() {
    let routes = post::app::api_route_inventory();

    assert!(routes.contains(&"/api/home"));
}
```

- [ ] **Step 4: Run tests and verify they fail**

Run:

```bash
cd post
cargo test
```

Expected: FAIL because `post::domain::home`, `ForumStore::home_page`, and `api_route_inventory` do not exist yet.

### Task 2: Add Homepage Domain Types

**Files:**
- Create: `post/src/domain/home.rs`
- Modify: `post/src/domain/mod.rs`

- [ ] **Step 1: Create `post/src/domain/home.rs`**

Create the module with these public types and deterministic seeded data:

```rust
use serde::{Deserialize, Serialize};

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
        filter_topics(seed_topics(), &query)
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
            query.tag.as_ref().is_none_or(|tag| {
                topic.tags.iter().any(|topic_tag| topic_tag.name == *tag)
            })
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
```

- [ ] **Step 2: Add the seed functions in the same module**

Append the seed functions with the exact design data:

```rust
fn seed_topics() -> Vec<HomeTopic> {
    vec![
        topic("leptos-release", TopicMarker::Pinned, "Leptos 0.6 发布：更快的编译、更小的体积、Signal 优化", "包含编译性能改进、Signal 内存优化、SSR 稳定性提升和迁移注意事项。", "公告", "blue", &["leptos"], 12, "3.2k", "张晨", "2 小时前", 980),
        topic("fullstack-guide", TopicMarker::Pinned, "新手指南：从 Axum + Leptos + SQLx 搭建全栈应用", "一步步搭建一个完整 CRUD 应用，包含认证、数据库操作和 SSR 渲染。", "教程", "green", &["axum", "sqlx", "+1"], 28, "7.8k", "李明", "5 小时前", 1200),
        topic("rules", TopicMarker::Locked, "站点规则与发帖规范（必读）", "请在发帖前阅读本站规则，帮助我们保持高质量的技术讨论环境。", "站务", "purple", &["规则"], 3, "9.1k", "管理员", "3 天前", 860),
        topic("server-function-sqlx", TopicMarker::Unread, "在 server function 中使用 SQLx 事务的最佳实践", "如何在 Leptos server function 中正确管理 SQLx 事务边界，避免连接泄漏。", "问题", "orange", &["leptos", "sqlx", "+1"], 7, "452", "wangxy", "1 小时前", 700),
        topic("flyio-deploy", TopicMarker::Unread, "Leptos + Axum 部署到 Fly.io 的完整流程", "分享 Leptos SSR 应用部署到 Fly.io 的配置、构建和环境变量设置。", "经验分享", "sky", &["leptos", "axum", "部署"], 5, "613", "DreamMao", "3 小时前", 690),
        topic("markdown-highlight", TopicMarker::Unread, "Markdown 渲染时如何高亮显示 Rust 代码？", "在 Leptos 中集成 pulldown-cmark 和 syntect，实现代码块高亮。", "问题", "orange", &["markdown", "rust", "+1"], 3, "289", "coderLin", "昨天 22:15", 520),
        topic("wasm-size", TopicMarker::Unread, "Leptos WebAssembly 包大小优化实践", "通过裁剪特性、增加缓存和 wasm-opt 减少包体积。", "经验分享", "sky", &["wasm", "leptos", "优化"], 9, "1.1k", "Skyline", "昨天 18:42", 760),
        topic("resources-repeat", TopicMarker::Read, "关于 resources! 宏在条件渲染下重复请求的问题", "当资源依赖发生变化且组件被重新挂载时，会触发重复请求，如何避免？", "问题", "orange", &["leptos", "resources"], 2, "163", "小林", "昨天 11:03", 300),
        topic("jsonb-config", TopicMarker::Read, "使用 PostgreSQL JSONB 存储配置的方案选择", "在配置灵活性和查询性能之间如何权衡？求推荐实践。", "讨论", "gray", &["postgresql", "jsonb"], 6, "342", "不二", "2 天前", 460),
        topic("signals-performance", TopicMarker::Read, "关于 Signals 与派生状态的性能陷阱", "在大型列表和复杂计算中，如何避免不必要的派生和内存分配。", "讨论", "gray", &["leptos", "signals"], 4, "276", "ChenKai", "2 天前", 430),
        topic("axum-body", TopicMarker::Read, "Axum 中间件处理 request body 的正确方式", "如何在不消耗 body 的情况下读取并复用请求体。", "问题", "orange", &["axum", "middleware"], 1, "198", "ZhangT", "2 天前", 280),
        topic("component-library", TopicMarker::Read, "从零实现一个简单的 Leptos 组件库", "记录组件库从脚手架到发布 crates.io 的全过程。", "经验分享", "sky", &["leptos", "组件库"], 3, "512", "Evan", "3 天前", 500),
    ]
}

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
```

- [ ] **Step 3: Expose the module**

Modify `post/src/domain/mod.rs`:

```rust
pub mod auth;
pub mod comments;
pub mod home;
pub mod notifications;
pub mod posts;
pub mod rbac;
pub mod reactions;
```

- [ ] **Step 4: Run the focused tests**

Run:

```bash
cd post
cargo test home_page_data_matches_dense_workbench_design_contract home_page_query_supports_tabs_filters_and_pagination_defaults
```

Expected: FAIL because `ForumStore::home_page` is not implemented.

### Task 3: Add Store-Level Homepage Query

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Import homepage domain types**

Update the domain import block:

```rust
domain::{
    auth::{Session, SessionUser},
    comments::{CommentNode, CreateCommentRequest},
    home::{dense_workbench_home, HomePageData, HomeQuery},
    posts::{CreatePostRequest, PostDetail, PostStatus, PostSummary},
    reactions::{FollowState, ToggleResult},
},
```

- [ ] **Step 2: Add `ForumStore::home_page`**

Add this method inside `impl ForumStore`:

```rust
pub fn home_page(
    &self,
    query: HomeQuery,
    current_user_id: Option<Uuid>,
) -> Result<HomePageData, ForumError> {
    if let Some(user_id) = current_user_id {
        let data = self.inner.read().map_err(|_| ForumError::Internal)?;
        if !data.users.contains_key(&user_id) {
            return Err(ForumError::Unauthorized);
        }
    }

    Ok(dense_workbench_home(query, current_user_id.is_some()))
}
```

- [ ] **Step 3: Run homepage data tests**

Run:

```bash
cd post
cargo test home_page_data_matches_dense_workbench_design_contract home_page_query_supports_tabs_filters_and_pagination_defaults
```

Expected: PASS.

### Task 4: Add `/api/home`

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`

- [ ] **Step 1: Import `Query` and homepage types**

Change the Axum imports:

```rust
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
```

Add the homepage import:

```rust
home::{HomePageData, HomeQuery, HomeSort, HomeTab, HomeTimeRange},
```

- [ ] **Step 2: Add the route**

Modify `routes`:

```rust
Router::new()
    .route("/api/home", get(home_page))
    .route("/api/login", post(login))
    .route("/api/posts", get(list_posts).post(create_post))
```

- [ ] **Step 3: Add query params and handler**

Add below `list_posts`:

```rust
#[derive(Clone, Debug, Default, Deserialize)]
struct HomeQueryParams {
    tab: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    time: Option<String>,
    sort: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
    user_id: Option<Uuid>,
}

impl From<HomeQueryParams> for HomeQuery {
    fn from(value: HomeQueryParams) -> Self {
        Self {
            tab: parse_tab(value.tab.as_deref()),
            category: value.category,
            tag: value.tag,
            time: parse_time_range(value.time.as_deref()),
            sort: parse_sort(value.sort.as_deref()),
            page: value.page.unwrap_or(1),
            page_size: value.page_size.unwrap_or(12),
        }
    }
}

async fn home_page(
    Extension(store): Extension<ForumStore>,
    Query(params): Query<HomeQueryParams>,
) -> Result<Json<HomePageData>, ApiError> {
    let user_id = params.user_id;
    Ok(Json(store.home_page(params.into(), user_id)?))
}

fn parse_tab(value: Option<&str>) -> HomeTab {
    match value {
        Some("hot") => HomeTab::Hot,
        Some("unanswered") => HomeTab::Unanswered,
        Some("following") => HomeTab::Following,
        _ => HomeTab::Latest,
    }
}

fn parse_time_range(value: Option<&str>) -> HomeTimeRange {
    match value {
        Some("today") => HomeTimeRange::Today,
        Some("week") => HomeTimeRange::Week,
        Some("month") => HomeTimeRange::Month,
        _ => HomeTimeRange::All,
    }
}

fn parse_sort(value: Option<&str>) -> HomeSort {
    match value {
        Some("created") => HomeSort::Created,
        Some("replies") => HomeSort::Replies,
        Some("views") => HomeSort::Views,
        Some("hot") => HomeSort::Hot,
        _ => HomeSort::LastReply,
    }
}
```

- [ ] **Step 4: Add API route inventory helper**

Add to `post/src/app.rs`:

```rust
pub fn api_route_inventory() -> Vec<&'static str> {
    vec![
        "/api/home",
        "/api/login",
        "/api/posts",
        "/api/posts/{post_id}",
        "/api/posts/{post_id}/comments",
        "/api/posts/{post_id}/like",
        "/api/posts/{post_id}/favorite",
        "/api/users/{user_id}/follow",
    ]
}
```

- [ ] **Step 5: Run API inventory test**

Run:

```bash
cd post
cargo test api_routes_include_homepage_aggregate_endpoint
```

Expected: PASS.

### Task 5: Bind Homepage UI to Typed Data

**Files:**
- Modify: `post/src/pages/home.rs`
- Modify: `post/src/app.rs`

- [ ] **Step 1: Replace local `ForumTopicRow` with domain types**

At the top of `post/src/pages/home.rs`, replace the local row struct with:

```rust
use leptos::prelude::*;

use crate::{
    components::PageShell,
    domain::home::{
        dense_workbench_home, HomeActiveAuthor, HomeAnnouncement, HomeCategory, HomeQuery, HomeTag,
        HomeTopic, TopicMarker,
    },
};
```

- [ ] **Step 2: Load homepage data in `HomePage`**

Replace the hardcoded `topics` vector with:

```rust
let home = dense_workbench_home(HomeQuery::default(), false);
let topics = home.topics.clone();
let categories = home.categories.clone();
let hot_tags = home.hot_tags.clone();
let announcements = home.announcements.clone();
let active_authors = home.active_authors.clone();
let pagination_label = home.pagination.label.clone();
```

- [ ] **Step 3: Render exactly four sidebar cards**

In the sidebar, use:

```rust
<aside class="side-stack">
    <CategoryPanel categories/>
    <TagPanel tags=hot_tags/>
    <AnnouncementPanel announcements/>
    <AuthorPanel authors=active_authors/>
</aside>
```

Remove the `NotificationPanel` component and remove its call site.

- [ ] **Step 4: Fix pagination display**

Replace the pager label:

```rust
<span>{pagination_label}</span>
```

- [ ] **Step 5: Update `TopicRow` to use `HomeTopic`**

Use this signature and marker mapping:

```rust
#[component]
fn TopicRow(topic: HomeTopic) -> impl IntoView {
    let marker_class = match topic.marker {
        TopicMarker::Pinned => "pin",
        TopicMarker::Locked => "lock",
        TopicMarker::Unread => "dot",
        TopicMarker::Read => "muted",
    };
    let tags = topic.tags.clone();

    view! {
        <a class="topic-row" href=format!("/posts/{}", topic.id)>
            <div class=format!("topic-marker {}", marker_class) aria-hidden="true"></div>
            <div class="topic-main">
                <h2>{topic.title}</h2>
                <p>{topic.summary}</p>
            </div>
            <div><span class=format!("badge badge-{}", topic.category.color)>{topic.category.name}</span></div>
            <div class="tag-list">
                {tags.into_iter().map(|tag| view! {
                    <span class="badge badge-soft">{tag.name}</span>
                }).collect_view()}
            </div>
            <div class="metric-cell">{topic.reply_count}</div>
            <div class="metric-cell">{topic.view_count_label}</div>
            <div class="last-reply">
                <span class="avatar-mini">{topic.last_reply.avatar_label}</span>
                <span><strong>{topic.last_reply.author}</strong><small>{topic.last_reply.time_label}</small></span>
            </div>
        </a>
    }
}
```

- [ ] **Step 6: Update sidebar components to accept data**

Use these component signatures:

```rust
#[component]
fn CategoryPanel(categories: Vec<HomeCategory>) -> impl IntoView

#[component]
fn TagPanel(tags: Vec<HomeTag>) -> impl IntoView

#[component]
fn AnnouncementPanel(announcements: Vec<HomeAnnouncement>) -> impl IntoView

#[component]
fn AuthorPanel(authors: Vec<HomeActiveAuthor>) -> impl IntoView
```

Each component should iterate over the provided vector and preserve the existing class names.

- [ ] **Step 7: Update homepage text inventory**

Modify `home_seed_text` in `post/src/app.rs`:

```rust
pub fn home_seed_text() -> &'static str {
    "Post Forum 首页 帖子 标签 用户 文档 活动 搜索帖子、标签、用户... 发布帖子 管理后台 通知 登录 最新 热门 未回复 关注 所有分类 所有标签 所有时间 主题 分类 标签 回复 查看 最后回复 显示 1-12 / 342 个主题 热门标签 公告 活跃作者"
}
```

- [ ] **Step 8: Run UI contract tests**

Run:

```bash
cd post
cargo test home_page_ui_inventory_matches_sidebar_and_pagination_contract dense_workbench_ui_exposes_prd_system_features
```

Expected: PASS.

### Task 6: Run Full Verification

**Files:**
- No new files.

- [ ] **Step 1: Run Rust tests**

Run:

```bash
cd post
cargo test
```

Expected: all tests pass.

- [ ] **Step 2: Run type check**

Run:

```bash
cd post
cargo check
```

Expected: build finishes without errors.

- [ ] **Step 3: Run Leptos build**

Run:

```bash
cd post
cargo leptos build
```

Expected: SSR and hydrate builds finish without errors.

- [ ] **Step 4: Check IDEA problems**

Use IDEA MCP problem inspection for these files:

- `post/src/domain/home.rs`
- `post/src/state.rs`
- `post/src/api.rs`
- `post/src/pages/home.rs`
- `post/src/app.rs`
- `post/tests/phase1_contract.rs`

Expected: no error-level problems. Warning-level problems can be reviewed after the first slice passes.

### Task 7: Commit the First Slice

**Files:**
- Stage only files changed by this slice.

- [ ] **Step 1: Review changed files**

Run:

```bash
git status --short
git diff -- post/src/domain/home.rs post/src/domain/mod.rs post/src/state.rs post/src/api.rs post/src/pages/home.rs post/src/app.rs post/tests/phase1_contract.rs
```

Expected: only homepage-system slice changes are included.

- [ ] **Step 2: Stage the slice**

Run:

```bash
git add post/src/domain/home.rs post/src/domain/mod.rs post/src/state.rs post/src/api.rs post/src/pages/home.rs post/src/app.rs post/tests/phase1_contract.rs
```

- [ ] **Step 3: Commit with the repository-required Chinese prefix**

Run:

```bash
git commit -m "[feature] 完成首页系统化数据支撑"
```

Expected: commit succeeds.

## Self-Review

- Spec coverage: This plan covers the homepage design landing requirements in `post/prd.md` section `4.1.1` and the homepage system support requirements in `4.1.2`.
- Explicit gaps: Full PostgreSQL persistence, Redis/NATS/RustFS/Elasticsearch production integration, WebSocket push, RBAC management screens, markdown upload pipeline, and full search result pages are not part of this first slice. They remain separate independently testable system slices.
- Placeholder scan: The plan contains no unresolved placeholders.
- Type consistency: `HomePageData`, `HomeQuery`, `HomeTopic`, and route names are consistent across tests, domain, store, API, and UI tasks.
