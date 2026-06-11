# Post Forum Phase28 RBAC Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move role normalization, permission resolution, role construction, role update, and built-in role delete protection out of `post/src/state.rs` into a focused RBAC service.

**Architecture:** `ForumStore` continues to own admin checks, duplicate checks, assigned-role checks, audit log insertion, and in-memory map writes. `RbacService` owns pure RBAC validation and transformation rules.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: RBAC Service Rules

**Files:**
- Create: `post/src/services/rbac.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::rbac::RbacService` to build a role from permission codes, apply a role update, reject unknown permissions, reject blank role codes, and reject built-in role deletion.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml rbac_service_normalizes_permissions_and_guards_builtin_roles`

Expected: FAIL because `post::services::rbac` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `RbacService` with `normalize_role_code`, `normalize_role_name`, `resolve_permissions`, `build_role`, `apply_role_update`, and `ensure_deletable_role`.

- [x] **Step 4: Wire store methods**

Update `ForumStore::create_role`, `update_role`, and `delete_role` to use `RbacService`, leaving state-dependent checks and audit logging in `state.rs`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml rbac_service_normalizes_permissions_and_guards_builtin_roles
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
