# Post Forum Phase37 Postgres Home Sidebar Repository Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add PostgreSQL repository support for homepage sidebar data: categories, hot tags, announcements, and active authors.

**Architecture:** Add `post::repositories::home` with explicit SQL strings, `FromRow` row structs, row-to-domain conversions, and async list methods. `AppState::home_page` keeps the dense homepage shell but replaces sidebar lists from PostgreSQL when a `PgPool` exists.

**Tech Stack:** Rust 2024, SQLx 0.9 `query_as`/`fetch_all`, PostgreSQL aggregate queries, Leptos SSR.

---

### Task 1: Home Sidebar Repository

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/src/repositories/home.rs`
- Modify: `post/src/repositories/mod.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Write the failing test**

Add `sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows` to `post/tests/phase1_contract.rs`. It should assert SQL fragments for categories, hot tags, announcements, and active authors and verify row conversions into `HomeCategory`, `HomeTag`, `HomeAnnouncement`, and `HomeActiveAuthor`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows
```

Expected: FAIL because `post::repositories::home` does not exist yet.

- [x] **Step 3: Write minimal implementation**

Create `post/src/repositories/home.rs`, register `pub mod home;`, and update `AppState::home_page` to call the repository in DB mode.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_home_sidebar_repository_contract_maps_homepage_sidebar_rows
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
