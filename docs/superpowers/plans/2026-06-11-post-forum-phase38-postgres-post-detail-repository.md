# Post Forum Phase38 Postgres Post Detail Repository Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add PostgreSQL-backed post detail reads and route `/api/posts/{post_id}` through `AppState`.

**Architecture:** Extend `post::repositories::posts` with a detail row and single-row query. `AppState::post_detail` delegates to `ForumStore` in demo mode and to `PostgresPostRepository` when a `PgPool` exists. The API detail handler extracts `AppState`.

**Tech Stack:** Rust 2024, SQLx 0.9 `query_as`/`fetch_optional`, PostgreSQL joins, Axum 0.8, Leptos SSR.

---

### Task 1: Post Detail Repository

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/repositories/posts.rs`
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`

- [x] **Step 1: Write the failing test**

Add `sqlx_post_detail_repository_contract_maps_post_detail_row` to `post/tests/phase1_contract.rs`. It should assert explicit SQL joins `posts`, `users`, `post_contents`, `categories`, `post_tags`, `tags`, use `where p.post_id = $1`, and verify `PostDetailRow` converts into `PostDetail`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_post_detail_repository_contract_maps_post_detail_row
```

Expected: FAIL because `PostDetailRow` and `post_detail_sql` do not exist yet.

- [x] **Step 3: Write minimal implementation**

Add `PostDetailRow`, `impl From<PostDetailRow> for PostDetail`, `post_detail_sql`, and `find_detail`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_post_detail_repository_contract_maps_post_detail_row
```

Expected: PASS.

- [x] **Step 5: Run full verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build
```

Expected: all commands pass.
