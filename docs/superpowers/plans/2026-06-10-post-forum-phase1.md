# Post Forum Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 `post` 目录下建立独立 Leptos 全栈论坛项目，并交付 PRD 第一阶段的可运行主干。

**Architecture:** 使用 Leptos SSR + Axum 作为全栈入口，SQLx + PostgreSQL 存储核心业务数据，DaisyUI/Tailwind 提供 UI。第一阶段先让业务链路以 PostgreSQL 为事实来源，Redis、NATS、RustFS、Elasticsearch 进入 Docker Compose 和模块边界，后续再接真实异步处理、对象存储和全文搜索。

**Tech Stack:** Rust、Leptos、leptos_axum、Axum、SQLx、PostgreSQL、DaisyUI、Tailwind CSS、Docker Compose。

---

## File Structure

- Create: `post/Cargo.toml`，Leptos SSR crate 配置。
- Create: `post/src/main.rs`，Axum/Leptos 服务入口。
- Create: `post/src/lib.rs`，前后端共享导出。
- Create: `post/src/app.rs`，Leptos 路由和页面壳。
- Create: `post/src/state.rs`，`AppState` 和配置。
- Create: `post/src/error.rs`，统一错误类型。
- Create: `post/src/domain/auth.rs`，注册、登录、session。
- Create: `post/src/domain/posts.rs`，帖子列表、详情、发布。
- Create: `post/src/domain/comments.rs`，评论和回复。
- Create: `post/src/domain/reactions.rs`，点赞、收藏、关注。
- Create: `post/src/domain/rbac.rs`，RBAC 权限判断。
- Create: `post/src/domain/notifications.rs`，通知数据结构和写入。
- Create: `post/src/pages/home.rs`，首页。
- Create: `post/src/pages/editor.rs`，发帖页。
- Create: `post/src/pages/post_detail.rs`，详情页。
- Create: `post/src/pages/login.rs`，登录注册页。
- Create: `post/src/pages/admin.rs`，管理端入口。
- Create: `post/src/components/mod.rs`，组件入口。
- Create: `post/style/main.css`，Tailwind/DaisyUI 入口。
- Create: `post/migrations/202606100001_phase1.sql`，第一阶段 schema。
- Create: `post/docker-compose.yml`，PostgreSQL、Redis、NATS、RustFS、Elasticsearch。
- Create: `post/.env.example`，本地配置示例。
- Create: `post/README.md`，启动、迁移、测试说明。
- Create: `post/tests/phase1_contract.rs`，核心业务契约测试。

## Task 1: Scaffold Leptos Project Shell

**Files:**
- Create: `post/Cargo.toml`
- Create: `post/src/main.rs`
- Create: `post/src/lib.rs`
- Create: `post/src/app.rs`
- Create: `post/src/state.rs`
- Create: `post/style/main.css`

- [x] **Step 1: Write failing smoke test**

Create `post/tests/phase1_contract.rs` with a test that expects the app shell to expose homepage navigation labels:

```rust
#[test]
fn app_shell_contract_lists_primary_routes() {
    let routes = post::app::primary_routes();
    assert!(routes.contains(&"/"));
    assert!(routes.contains(&"/posts/new"));
    assert!(routes.contains(&"/login"));
    assert!(routes.contains(&"/admin"));
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cd post && cargo test app_shell_contract_lists_primary_routes`

Expected: FAIL because `post::app::primary_routes` is not defined.

- [x] **Step 3: Implement minimal app shell**

Create `post/src/app.rs` with:

```rust
pub fn primary_routes() -> Vec<&'static str> {
    vec!["/", "/posts/new", "/login", "/admin"]
}
```

Expose it from `post/src/lib.rs`:

```rust
pub mod app;
```

- [x] **Step 4: Run test to verify it passes**

Run: `cd post && cargo test app_shell_contract_lists_primary_routes`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add post docs/superpowers
git commit -m "[feature] 初始化论坛项目壳"
```

## Task 2: Database Schema Contract

**Files:**
- Create: `post/migrations/202606100001_phase1.sql`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing schema test**

Append:

```rust
#[test]
fn phase1_schema_contains_prd_core_tables() {
    let schema = include_str!("../migrations/202606100001_phase1.sql");
    for table in [
        "users",
        "sessions",
        "roles",
        "permissions",
        "posts",
        "post_contents",
        "comments",
        "post_likes",
        "post_favorites",
        "follows",
        "notifications",
        "announcements",
        "files",
        "reports",
        "audit_logs",
    ] {
        assert!(schema.contains(&format!("create table {table}")), "missing {table}");
    }
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cd post && cargo test phase1_schema_contains_prd_core_tables`

Expected: FAIL because the migration is missing.

- [x] **Step 3: Create schema**

Create migration with explicit tables, foreign keys, unique constraints for likes/favorites/follows, and status checks for posts/comments/users.

- [x] **Step 4: Run test to verify it passes**

Run: `cd post && cargo test phase1_schema_contains_prd_core_tables`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add post/migrations post/tests
git commit -m "[feature] 增加论坛核心数据模型"
```

## Task 3: Domain Contract Types

**Files:**
- Create: `post/src/domain/mod.rs`
- Create: `post/src/domain/auth.rs`
- Create: `post/src/domain/posts.rs`
- Create: `post/src/domain/comments.rs`
- Create: `post/src/domain/reactions.rs`
- Create: `post/src/domain/rbac.rs`
- Create: `post/src/domain/notifications.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing domain test**

Append:

```rust
#[test]
fn post_summary_contract_matches_homepage_requirements() {
    let summary = post::domain::posts::PostSummary::sample();
    assert!(!summary.title.is_empty());
    assert!(!summary.author_name.is_empty());
    assert!(summary.view_count >= 0);
    assert!(summary.comment_count >= 0);
    assert!(summary.like_count >= 0);
    assert!(summary.favorite_count >= 0);
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cd post && cargo test post_summary_contract_matches_homepage_requirements`

Expected: FAIL because `domain::posts::PostSummary` is not defined.

- [x] **Step 3: Implement minimal shared domain types**

Create `PostSummary`, `PostDetail`, `CommentNode`, `Notification`, `Permission`, `SessionUser` structs with serializable fields required by PRD first-stage pages.

- [x] **Step 4: Run test to verify it passes**

Run: `cd post && cargo test post_summary_contract_matches_homepage_requirements`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add post/src/domain post/tests
git commit -m "[feature] 定义论坛领域契约"
```

## Task 4: Infrastructure Configuration

**Files:**
- Create: `post/docker-compose.yml`
- Create: `post/.env.example`
- Create: `post/README.md`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing infrastructure test**

Append:

```rust
#[test]
fn compose_declares_required_prd_services() {
    let compose = include_str!("../docker-compose.yml");
    for service in ["postgres", "redis", "nats", "rustfs", "elasticsearch"] {
        assert!(compose.contains(&format!("{service}:")), "missing {service}");
    }
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cd post && cargo test compose_declares_required_prd_services`

Expected: FAIL because compose is missing required services.

- [x] **Step 3: Add Docker Compose and README**

Define all PRD services with persistent volumes, health checks where images support them, stable local ports, and `.env.example` defaults.

- [x] **Step 4: Run test to verify it passes**

Run: `cd post && cargo test compose_declares_required_prd_services`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add post/docker-compose.yml post/.env.example post/README.md post/tests
git commit -m "[feature] 增加论坛本地依赖环境"
```

## Task 5: First Usable Leptos/DaisyUI UI

**Files:**
- Modify: `post/src/app.rs`
- Create: `post/src/pages/home.rs`
- Create: `post/src/pages/editor.rs`
- Create: `post/src/pages/post_detail.rs`
- Create: `post/src/pages/login.rs`
- Create: `post/src/pages/admin.rs`
- Create: `post/src/components/mod.rs`
- Modify: `post/style/main.css`

- [x] **Step 1: Write failing UI contract test**

Append:

```rust
#[test]
fn home_seed_content_exposes_forum_workflow() {
    let text = post::app::home_seed_text();
    for required in ["推荐", "最新", "热门", "发布帖子", "评论", "管理端"] {
        assert!(text.contains(required), "missing {required}");
    }
}
```

- [x] **Step 2: Run test to verify it fails**

Run: `cd post && cargo test home_seed_content_exposes_forum_workflow`

Expected: FAIL because `home_seed_text` is not defined.

- [x] **Step 3: Implement UI seed and pages**

Implement homepage shell, post card list, editor shell, detail shell, login shell, and admin shell with DaisyUI classes. All visible controls must map to PRD workflows even when deeper backends are still being wired.

- [x] **Step 4: Run test and build**

Run: `cd post && cargo test home_seed_content_exposes_forum_workflow`

Expected: PASS.

Run: `cd post && cargo check`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add post/src post/style post/tests
git commit -m "[feature] 实现论坛首屏和核心页面壳"
```

## Task 6: Verification

**Files:**
- Modify only if verification finds errors.

- [x] **Step 1: Run Rust verification**

Run: `cd post && cargo test`

Expected: PASS.

Run: `cd post && cargo check`

Expected: PASS.

- [x] **Step 2: Run Leptos build when toolchain is available**

Run: `cd post && cargo leptos build`

Expected: PASS, or document the missing toolchain/component if the local machine lacks it.

- [x] **Step 3: Inspect IDEA problems**

Use IDEA MCP against project path `/Users/mah2/project/rust-learn`, prioritizing error-level problems under `post/`.

- [ ] **Step 4: Commit fixes**

```bash
git add post docs/superpowers
git commit -m "[update] 修复论坛项目验证问题"
```

## Self-Review

- Spec coverage: Task 1-5 cover independent project, Leptos shell, schema, core domain contracts, JSON API, Docker Compose dependencies, README and first usable UI. Remaining PRD depth is explicitly staged in the design document.
- Placeholder scan: no implementation placeholder is intentionally left in the plan; later-stage PRD work is scoped as future phases, not an unspecified hole in this phase.
- Type consistency: tests reference `post::app` and `post::domain::posts`; those modules are created before use.
