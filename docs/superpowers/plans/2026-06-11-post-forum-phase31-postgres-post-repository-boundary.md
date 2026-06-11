# Post Forum Phase31 PostgreSQL Post Repository Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the first SQLx/PostgreSQL repository boundary for homepage post summaries so the system can start moving from `ForumStore` in-memory data toward real persistence.

**Architecture:** Keep existing API behavior on `ForumStore` for now. Add `post/src/repositories/posts.rs` with an explicit SQL query, a `FromRow` row type, row-to-domain mapping, and an async `list_published_summaries` method using `PgPool`.

**Tech Stack:** Rust, SQLx 0.9 PostgreSQL, `PgPool`, `query_as`, `FromRow`, Leptos SSR.

---

### Task 1: PostgreSQL Post Summary Repository

**Files:**
- Create: `post/src/repositories/mod.rs`
- Create: `post/src/repositories/posts.rs`
- Modify: `post/src/lib.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a repository-level test that references `post::repositories::posts::PostgresPostRepository`, checks the SQL query uses explicit columns and required joins for homepage summaries, and verifies row-to-`PostSummary` mapping.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml sqlx_post_repository_contract_maps_homepage_post_rows`

Expected: FAIL because `post::repositories` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `PostgresPostRepository`, `PostSummaryRow`, `published_summaries_sql`, and `list_published_summaries(&PgPool, limit, offset)` using `sqlx::query_as`.

- [x] **Step 4: Export repository module**

Expose `repositories` from `post/src/lib.rs` under the SSR feature.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml sqlx_post_repository_contract_maps_homepage_post_rows
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
