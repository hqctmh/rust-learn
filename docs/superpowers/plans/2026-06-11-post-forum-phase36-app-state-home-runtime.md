# Post Forum Phase36 App State Home Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route `/api/home` through `AppState` and let PostgreSQL mode replace the homepage topic list with persisted published posts.

**Architecture:** `ForumStore::home_page` remains the dense demo homepage source. `AppState::home_page` delegates to it when no database is configured; when a `PgPool` exists, it builds the same dense homepage shell and replaces `topics` with `PostgresPostRepository` summaries converted to `HomeTopic`.

**Tech Stack:** Rust 2024, Axum 0.8 `Extension`, SQLx 0.9, PostgreSQL, Leptos SSR.

---

### Task 1: AppState Home Runtime

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/domain/home.rs`
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`

- [x] **Step 1: Write the failing test**

Add `app_state_home_runtime_supports_demo_fallback` to `post/tests/phase1_contract.rs`. It should call `AppState::home_page(HomeQuery::default(), None).await` and assert the design homepage still has 12 topics, sidebar categories, and pagination.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_home_runtime_supports_demo_fallback
```

Expected: FAIL because `AppState::home_page` does not exist yet.

- [x] **Step 3: Write minimal implementation**

Add `HomeTopic::from_post_summary` conversion helper, `AppState::home_page`, and update `/api/home` to extract `AppState`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_home_runtime_supports_demo_fallback
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
