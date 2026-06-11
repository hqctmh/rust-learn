# Post Forum User Management And Audit Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real user management and audit logging for PRD 5.3 and 5.9, including disable/enable users, role changes, and operation logs.

**Architecture:** Keep `SessionUser` small and store user governance state in `ForumStore` maps. Add a focused `user_admin` domain module for admin-facing rows, role updates, and audit entries. Admin mutations record audit entries that are also exposed through API and dashboard data.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, time, existing contract tests.

---

### Task 1: User Admin Domain and Tests

**Files:**
- Create: `post/src/domain/user_admin.rs`
- Modify: `post/src/domain/mod.rs`
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add failing workflow test**

Add `user_admin_contract_supports_disable_enable_roles_and_audit_logs` asserting:
- admin can list users and see normal user rows
- non-admin users cannot list users
- admin can disable a user
- disabled users cannot log in
- admin can enable the user again
- admin can assign roles
- each mutation writes audit entries with actor, action, object type, object ID, IP, User-Agent, and timestamp

- [ ] **Step 2: Add failing route test**

Add `user_admin_routes_are_registered` covering:
- `GET /api/admin/users`
- `POST /api/admin/users/{user_id}/disable`
- `POST /api/admin/users/{user_id}/enable`
- `POST /api/admin/users/{user_id}/roles`
- `GET /api/admin/audit-logs`

- [ ] **Step 3: Implement user admin types**

Create `AdminUserRow`, `UpdateUserRolesRequest`, `AuditLogEntry`, and `AuditContext`.

### Task 2: Store Behavior

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Add governance state**

Add `disabled_users: HashSet<Uuid>`, `user_roles: HashMap<Uuid, Vec<String>>`, and `audit_logs: Vec<AuditLogEntry>` to `ForumData`.

- [ ] **Step 2: Enforce disabled login**

Update `login` to reject disabled users.

- [ ] **Step 3: Implement user admin methods**

Add `admin_users`, `disable_user`, `enable_user`, `update_user_roles`, and `audit_logs`.

- [ ] **Step 4: Record audit logs**

Each mutation records action, target type, target ID, before/after summaries, IP, User-Agent, and operation time.

### Task 3: API and Admin UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/domain/admin.rs`
- Modify: `post/src/pages/admin.rs`

- [ ] **Step 1: Register user admin APIs**

Wire user list, disable, enable, role update, and audit list routes.

- [ ] **Step 2: Update route inventory**

Add the new API paths to `api_route_inventory`.

- [ ] **Step 3: Update admin dashboard**

Add user management rows and real audit fields to the dashboard UI surface.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`.

- [ ] **Step 2: Focused tests**

Run: `cargo test user_admin --test phase1_contract`.

- [ ] **Step 3: Full tests and builds**

Run: `cargo test`, `cargo check`, and `cargo leptos build`.

- [ ] **Step 4: IDEA errors**

Use IDEA MCP `get_file_problems(errorsOnly=true)` on changed Rust files and fix error-level findings.
