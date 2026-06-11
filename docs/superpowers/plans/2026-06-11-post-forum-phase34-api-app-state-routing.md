# Post Forum Phase34 API App State Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Route HTTP API authentication through `AppState` so configured PostgreSQL auth can be used by real endpoints.

**Architecture:** `api::routes` accepts `AppState` and layers both `AppState` and its `ForumStore` into Axum request extensions. Existing non-auth handlers can continue extracting `ForumStore`; auth handlers extract `AppState` and call its async runtime methods.

**Tech Stack:** Rust 2024, Axum 0.8 `Extension`, SQLx PgPool, Leptos SSR.

---

### Task 1: API Routes Use AppState

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/api.rs`
- Modify: `post/src/main.rs`

- [x] **Step 1: Write the failing test**

Add `api_routes_accept_app_state_runtime` to `post/tests/phase1_contract.rs`. The test should construct `AppState { db: None, forum: ForumStore::seeded() }` and pass it to `post::api::routes(state)`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml api_routes_accept_app_state_runtime
```

Expected: FAIL because `api::routes` still expects `ForumStore`.

- [x] **Step 3: Write minimal implementation**

Change `api::routes` to accept `AppState`, layer `Extension(state.forum.clone())` and `Extension(state)`, and update auth handlers to extract `AppState`. Change `main.rs` to construct `AppState`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml api_routes_accept_app_state_runtime
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
