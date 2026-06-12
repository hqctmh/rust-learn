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

**Current verification evidence (2026-06-12):**
- Status: Completed and re-verified on 2026-06-12.
- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 96 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

### Task 2: Protected APIs Require Explicit Actor Identity

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/api.rs`

- [x] **Step 1: Write the failing test**

Add `protected_api_handlers_require_explicit_actor_identity` to assert protected API handlers share an explicit actor identity guard and do not silently impersonate the demo user.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml protected_api_handlers_require_explicit_actor_identity
```

Expected: FAIL because `api.rs` still falls back to `state.forum.demo_user().user_id`.

- [x] **Step 3: Write minimal implementation**

Add `require_user_id` and route protected API handlers through it. Keep public read endpoints public, but require explicit `user_id` on write, admin, notification, upload, and personal-action APIs.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml protected_api_handlers_require_explicit_actor_identity
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

**Current verification evidence (2026-06-12):**
- Status: Completed and verified on 2026-06-12.
- `cargo test --manifest-path post/Cargo.toml protected_api_handlers_require_explicit_actor_identity`: PASS, 1 passed, 95 filtered out.
- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 96 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.

### Task 3: Protected APIs Resolve Actor From Session

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/api.rs`
- Modify: `post/src/domain/notifications.rs`

- [x] **Step 1: Write the failing test**

Add `protected_api_handlers_resolve_actor_from_session_id` to assert protected API identity params accept `session_id`, resolve it through `AppState::current_session`, and route protected handlers through the session-aware guard.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml protected_api_handlers_resolve_actor_from_session_id
```

Expected: FAIL because protected API identity params only accept raw `user_id`.

- [x] **Step 3: Write minimal implementation**

Add `ActorIdentity` plus `require_actor_id`. Add optional `session_id` support to admin, author, notification query params and user action JSON bodies. Resolve `session_id` through `AppState::current_session(session_id).await` before falling back to the existing explicit `user_id` development path.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml protected_api_handlers_resolve_actor_from_session_id
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

**Current verification evidence (2026-06-12):**
- Status: Completed and verified on 2026-06-12.
- `cargo test --manifest-path post/Cargo.toml protected_api_handlers_resolve_actor_from_session_id`: PASS, 1 passed, 96 filtered out.
- `cargo fmt --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 97 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
