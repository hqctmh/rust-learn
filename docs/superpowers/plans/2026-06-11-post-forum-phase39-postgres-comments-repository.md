# Post Forum Phase39 Postgres Comments Repository Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add PostgreSQL-backed comment list reads and route `/api/posts/{post_id}/comments` through `AppState`.

**Architecture:** Add `post::repositories::comments` with explicit SQL and a flat-row to `CommentNode` tree assembler. `AppState::comments_for_post` delegates to `ForumStore` in demo mode and to `PostgresCommentRepository` when a `PgPool` exists. The API comment-list handler extracts `AppState`.

**Tech Stack:** Rust 2024, SQLx 0.9 `query_as`/`fetch_all`, PostgreSQL joins, Axum 0.8, Leptos SSR.

---

### Task 1: Comments Repository

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/src/repositories/comments.rs`
- Modify: `post/src/repositories/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`

- [x] **Step 1: Write the failing test**

Add `sqlx_comment_repository_contract_maps_comment_tree_rows` to `post/tests/phase1_contract.rs`. It should assert explicit SQL joins `comments`, `users`, and `posts`, use `where c.post_id = $1`, and verify flat rows build a top-level comment with one reply and mask deleted comments.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_comment_repository_contract_maps_comment_tree_rows
```

Expected: FAIL because `post::repositories::comments` does not exist yet.

- [x] **Step 3: Write minimal implementation**

Create `CommentRow`, `PostgresCommentRepository::comments_for_post_sql`, `build_comment_tree`, and `list_for_post`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_comment_repository_contract_maps_comment_tree_rows
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

**Current verification evidence (2026-06-11):**
- Status: Completed and re-verified on 2026-06-11.
- `cargo test --manifest-path post/Cargo.toml sqlx_comment_repository_contract_maps_comment_tree_rows`: PASS, 1 passed, 82 filtered out.
- `cargo test --manifest-path post/Cargo.toml sqlx_comment_repository_contract_maps_comment_tree_rows`: PASS, 1 passed.
- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 79 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
