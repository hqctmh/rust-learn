# Post Forum Phase32 Postgres Auth Repository Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a SQLx-backed repository boundary for users and sessions so auth can move out of the in-memory `ForumStore`.

**Architecture:** Keep `state.rs` as the current demo/store facade for now, but add a focused `post::repositories::auth` module for PostgreSQL user/session row mapping and SQL entry points. The repository exposes explicit SQL strings for contract tests and async helpers that use `PgPool`, `query_as`, `fetch_optional`, and `execute`.

**Tech Stack:** Rust 2024, SQLx 0.9, PostgreSQL, `uuid`, `time::OffsetDateTime`, Leptos SSR feature gating.

---

### Task 1: User And Session Repository Contract

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Create: `post/src/repositories/auth.rs`
- Modify: `post/src/repositories/mod.rs`

- [x] **Step 1: Write the failing test**

Add `sqlx_auth_repository_contract_maps_users_and_sessions` to `post/tests/phase1_contract.rs`. The test must assert that user lookup, user insert, session insert, session lookup, and session delete SQL use explicit columns and schema names from `post/migrations/202606100001_phase1.sql`. It must also assert that row structs convert into `SessionUser` and `Session`.

- [x] **Step 2: Run test to verify it fails**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_auth_repository_contract_maps_users_and_sessions
```

Expected: FAIL because `post::repositories::auth` does not exist yet.

- [x] **Step 3: Write minimal implementation**

Create `post/src/repositories/auth.rs` with `UserAuthRow`, `SessionAuthRow`, and `PostgresAuthRepository`. Add `pub mod auth;` to `post/src/repositories/mod.rs`.

- [x] **Step 4: Run targeted test to verify it passes**

Run:

```bash
cargo test --manifest-path post/Cargo.toml sqlx_auth_repository_contract_maps_users_and_sessions
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
