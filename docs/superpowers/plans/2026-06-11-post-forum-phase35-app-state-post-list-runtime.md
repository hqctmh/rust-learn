# Post Forum Phase35 App State Post List Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route `/api/posts` list reads through `AppState` so configured PostgreSQL persistence can serve published post summaries.

**Architecture:** `AppState` gets an async `list_posts` method. With `db: Some(PgPool)` it calls `PostgresPostRepository::list_published_summaries`; with `db: None` it delegates to the existing in-memory `ForumStore`.

**Tech Stack:** Rust 2024, Axum 0.8 `Extension`, SQLx 0.9 `query_as`/`fetch_all`, PostgreSQL, Leptos SSR.

---

### Task 1: AppState Post List Runtime

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`

- [x] **Step 1: Write the failing test**

Add `app_state_post_list_runtime_supports_demo_fallback` to `post/tests/phase1_contract.rs`. It should construct `AppState { db: None, forum: ForumStore::seeded() }`, call `list_posts().await`, and assert it returns the seeded published post.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_post_list_runtime_supports_demo_fallback
```

Expected: FAIL because `AppState::list_posts` does not exist yet.

- [x] **Step 3: Write minimal implementation**

Add `AppState::list_posts` and update the `/api/posts` GET handler to extract `AppState` and return `Result<Json<Vec<PostSummary>>, ApiError>`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_post_list_runtime_supports_demo_fallback
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
