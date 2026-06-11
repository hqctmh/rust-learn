# Post Forum Phase30 User Admin Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move user administration projection, role normalization, self-disable guard, audit snapshot, and audit log construction rules out of `post/src/state.rs` into a focused user admin service.

**Architecture:** `ForumStore` continues to own admin checks, user/role existence checks, post/comment counting, disabled-user storage, user-role map writes, and audit log storage. `UserAdminService` owns pure transformations for user admin rows, audit snapshots, audit entries, and normalized role inputs.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: User Admin Service Rules

**Files:**
- Create: `post/src/services/user_admin.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::user_admin::UserAdminService` to guard self-disable, normalize roles, build admin rows, build audit snapshots, and build audit log entries.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml user_admin_service_builds_rows_roles_and_audit_entries`

Expected: FAIL because `post::services::user_admin` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `UserAdminService` with `ensure_not_self_disable`, `normalize_roles`, `admin_user_row`, `audit_snapshot`, and `build_audit_log`.

- [x] **Step 4: Wire store helpers and methods**

Update `ForumStore::disable_user`, `update_user_roles`, `admin_user_row`, `user_audit_snapshot`, and `push_audit_log` to use `UserAdminService`, leaving state-dependent lookups and writes in `state.rs`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml user_admin_service_builds_rows_roles_and_audit_entries
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
