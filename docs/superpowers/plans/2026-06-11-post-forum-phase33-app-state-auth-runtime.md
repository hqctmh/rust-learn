# Post Forum Phase33 App State Auth Runtime Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `AppState` an auth runtime that can use PostgreSQL repositories when a `PgPool` is configured while preserving the in-memory demo fallback.

**Architecture:** `ForumStore` remains the in-memory demo store. `AppState` becomes the runtime facade for auth methods and decides whether to call `PostgresAuthRepository` or delegate to `ForumStore`.

**Tech Stack:** Rust 2024, SQLx 0.9, PostgreSQL lazy pool, Axum-compatible application state, Leptos SSR.

---

### Task 1: AppState Auth Runtime

**Files:**
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add `app_state_auth_runtime_supports_postgres_mode_and_demo_fallback` to `post/tests/phase1_contract.rs`. The test should create `AppState` with `db: None` and prove async login/register/current-session/logout work through the demo store. It should also create a lazy `PgPool` and prove `AppState::uses_postgres_auth()` reports true.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_auth_runtime_supports_postgres_mode_and_demo_fallback
```

Expected: FAIL because `AppState` does not expose the async auth runtime methods yet.

- [x] **Step 3: Write minimal implementation**

Add `uses_postgres_auth`, `login`, `register`, `current_session`, and `logout` methods on `AppState`. PostgreSQL mode should use `PostgresAuthRepository`; demo mode should delegate to `ForumStore`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml app_state_auth_runtime_supports_postgres_mode_and_demo_fallback
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
