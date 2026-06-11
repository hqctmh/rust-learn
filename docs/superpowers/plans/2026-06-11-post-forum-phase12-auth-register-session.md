# Post Forum Registration And Session Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add explicit registration, logout, and current-session APIs plus a registration page so PRD user auth flows are supported beyond the demo login.

**Architecture:** Reuse existing `SessionUser`, `RegisterRequest`, and `Session`. Keep in-memory storage but separate explicit registration from login, add duplicate username validation, preserve current login demo behavior, and expose session lifecycle APIs through Axum and route inventory.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, existing contract tests.

---

### Task 1: Auth Workflow Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add failing auth workflow test**

Add `auth_contract_supports_register_current_session_and_logout` asserting:
- register creates a session and user with nickname
- duplicate username registration fails
- current session lookup returns the session
- logout removes the session
- current session lookup after logout fails
- disabled users cannot register with the same username

- [ ] **Step 2: Add failing route test**

Add `auth_routes_are_registered` covering:
- `/register`
- `/api/register`
- `/api/logout`
- `/api/session/{session_id}`

### Task 2: Store Auth Behavior

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Implement register**

Add `register(&self, RegisterRequest) -> Result<Session, ForumError>` with validation, duplicate username check, member role insertion, and session creation.

- [ ] **Step 2: Implement current session and logout**

Add `current_session(session_id)` and `logout(session_id)`; logout should remove the session and return the removed session.

### Task 3: API and Page

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Create: `post/src/pages/register.rs`
- Modify: `post/src/pages/mod.rs`
- Modify: `post/src/pages/login.rs`

- [ ] **Step 1: Register API routes**

Wire `/api/register`, `/api/logout`, and `/api/session/{session_id}`.

- [ ] **Step 2: Add registration page**

Create a register page matching the current auth panel style with username, nickname, password, confirm password, and login link.

- [ ] **Step 3: Update route inventory and navigation**

Add `/register` to `primary_routes`, route it in `App`, export page module, and turn the login page register button into a link.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`.

- [ ] **Step 2: Focused tests**

Run: `cargo test auth_ --test phase1_contract`.

- [ ] **Step 3: Full tests and builds**

Run: `cargo test`, `cargo check`, and `cargo leptos build`.

- [ ] **Step 4: IDEA errors**

Use IDEA MCP `get_file_problems(errorsOnly=true)` on changed Rust files and fix error-level findings.
