# Post Forum Phase 16 RBAC Role Permission Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Complete the admin role and permission management capabilities required by `post/prd.md` sections 5.2, 5.3, 11.2, 12, 13, and 16.

**Architecture:** Promote the existing static RBAC permission list into store-backed role data. Seed default roles (`admin`, `member`, `moderator`, `operator`) so existing user-role tests remain valid. Add role CRUD methods with audit logging and expose thin Axum handlers under `/api/admin/roles` and `/api/admin/permissions`.

**Tech Stack:** Rust, Leptos, Axum 0.8 routing/extractors, serde, existing `ForumStore`, `ForumError`, `AuditContext`, `Role`, and `Permission` patterns.

---

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing RBAC behavior test**

Add `rbac_contract_supports_role_and_permission_management` proving:
- normal users cannot list roles or permissions;
- admin can list permissions and sees `role:view`, `role:create`, `role:update`, `role:delete`, `permission:view`;
- admin sees default `admin`, `member`, `moderator`, `operator` roles;
- admin can create a new role with permissions;
- duplicate role create is rejected;
- admin can update the role name and permissions;
- deleting an assigned role is rejected;
- deleting an unassigned role succeeds.

- [x] **Step 2: Write failing route test**

Add `rbac_routes_are_registered` asserting:
- `/api/admin/roles`
- `/api/admin/roles/{role_code}/update`
- `/api/admin/roles/{role_code}/delete`
- `/api/admin/permissions`

- [x] **Step 3: Run targeted tests and verify red**

Run:

```bash
cargo test rbac_ --test phase1_contract
```

Expected: fail because request structs, store methods, and routes do not exist yet.

### Task 2: Domain and Store

**Files:**
- Modify: `post/src/domain/rbac.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Add request structs and richer permissions**

Add:
- `CreateRoleRequest`
- `UpdateRoleRequest`

Extend `admin_permissions()` with the PRD role and permission codes.

- [x] **Step 2: Add store role map**

Add `roles: HashMap<String, Role>` to `ForumData` and seed default roles.

- [x] **Step 3: Implement RBAC methods**

Add:
- `ForumStore::list_roles(admin_id)`
- `ForumStore::list_permissions(admin_id)`
- `ForumStore::create_role(admin_id, request)`
- `ForumStore::update_role(admin_id, role_code, request)`
- `ForumStore::delete_role(admin_id, role_code)`

Rules:
- admin-only;
- role code normalized to lowercase;
- role code/name cannot be empty;
- requested permission codes must exist;
- duplicate role create returns conflict;
- assigned roles cannot be deleted;
- role changes write audit logs.

- [x] **Step 4: Run behavior test and verify green**

Run:

```bash
cargo test rbac_contract --test phase1_contract
```

Expected: pass.

### Task 3: API and UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/pages/admin.rs`

- [x] **Step 1: Add API routes and handlers**

Add:
- `GET/POST /api/admin/roles`
- `POST /api/admin/roles/{role_code}/update`
- `POST /api/admin/roles/{role_code}/delete`
- `GET /api/admin/permissions`

Use the existing `AdminDashboardQueryParams` for admin identity and `AuditContext` from request bodies where mutations need audit data.

- [x] **Step 2: Add inventory entries**

Add the four API route strings to `api_route_inventory()`.

- [x] **Step 3: Surface role/permission sections in admin page**

Ensure the admin page visibly contains role and permission management tables/actions, not only menu labels.

- [x] **Step 4: Run route test and verify green**

Run:

```bash
cargo test rbac_routes_are_registered --test phase1_contract
```

Expected: pass.

### Task 4: Verification

**Files:**
- All touched files.

- [x] **Step 1: Format**

Run:

```bash
cargo fmt
```

- [x] **Step 2: Full verification**

Run:

```bash
cargo test
cargo check
cargo leptos build
```

- [x] **Step 3: IDEA error inspection**

Check errors only for:
- `post/src/domain/rbac.rs`
- `post/src/state.rs`
- `post/src/api.rs`
- `post/src/app.rs`
- `post/src/pages/admin.rs`
- `post/tests/phase1_contract.rs`

Expected: no errors.
