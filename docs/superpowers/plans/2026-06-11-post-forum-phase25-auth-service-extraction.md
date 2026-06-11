# Post Forum Phase25 Auth Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move login, registration, password, and session construction rules out of `post/src/state.rs` into a focused auth service.

**Architecture:** `ForumStore` remains the in-memory application facade for this phase, but pure auth rules live in `post/src/services/auth.rs`. The store keeps repository-like responsibilities such as duplicate lookup, disabled-user checks, and session persistence.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: Auth Service Rules

**Files:**
- Create: `post/src/services/auth.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::auth::AuthService` to normalize login/register input, build users, validate password matches, and build seven-day sessions.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml auth_service_normalizes_credentials_and_builds_sessions`

Expected: FAIL because `post::services::auth` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `AuthService` with normalized request structs, credential validation, registered-user construction, login-user construction, password match validation, and session construction.

- [x] **Step 4: Wire store methods**

Update `ForumStore::login`, `ForumStore::register`, and `ForumStore::current_session` to use `AuthService` for pure business rules while leaving in-memory reads/writes inside `state.rs`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml auth_service_normalizes_credentials_and_builds_sessions
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
