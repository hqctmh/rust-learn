# Post Forum Taxonomy Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add category and tag management so homepage taxonomy, filtering chips, and admin CRUD/merge requirements are backed by real system behavior.

**Architecture:** Introduce a focused `taxonomy` domain module for category/tag contracts. Store category and tag records in the existing in-memory `ForumStore`, enforce admin-only mutation, project enabled taxonomy records into homepage sidebar data, and expose public/admin Axum JSON APIs.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, existing contract tests.

---

### Task 1: Taxonomy Domain and Tests

**Files:**
- Create: `post/src/domain/taxonomy.rs`
- Modify: `post/src/domain/mod.rs`
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add failing category/tag workflow test**

Add `taxonomy_contract_supports_admin_category_and_tag_management` asserting:
- seeded categories match the Dense Workbench design counts
- non-admin users cannot create categories
- admin can create/update/disable a category
- admin can create/update/merge/delete tags
- merged tag count moves from source to target
- homepage category and hot tag modules reflect enabled managed taxonomy

- [ ] **Step 2: Add failing route inventory test**

Add `taxonomy_routes_are_registered` covering:
- `GET /api/categories`
- `GET /api/tags`
- `GET /api/admin/categories`
- `POST /api/admin/categories`
- `POST /api/admin/categories/{category_id}/update`
- `POST /api/admin/categories/{category_id}/disable`
- `GET /api/admin/tags`
- `POST /api/admin/tags`
- `POST /api/admin/tags/{tag_id}/update`
- `POST /api/admin/tags/{tag_id}/merge`
- `POST /api/admin/tags/{tag_id}/delete`

- [ ] **Step 3: Implement taxonomy types**

Create `CategoryItem`, `TagItem`, `CreateCategoryRequest`, `UpdateCategoryRequest`, `CreateTagRequest`, `UpdateTagRequest`, and `MergeTagRequest` with serde derives and validation.

### Task 2: Store Behavior

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Add store fields and seeds**

Add `categories: HashMap<Uuid, CategoryItem>` and `tags: HashMap<Uuid, TagItem>` to `ForumData`. Seed the exact homepage design categories and hot tags.

- [ ] **Step 2: Implement public taxonomy reads**

Add `public_categories` and `public_tags`, sorted by display order/count and filtered to enabled records.

- [ ] **Step 3: Implement admin mutations**

Add `create_category`, `update_category`, `disable_category`, `create_tag`, `update_tag`, `merge_tag`, and `delete_tag`, all guarded by admin permission.

- [ ] **Step 4: Project taxonomy into homepage**

Make `home_page` replace static category/tag modules with store-managed enabled taxonomy data.

### Task 3: API and Admin UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/domain/admin.rs`
- Modify: `post/src/pages/admin.rs`

- [ ] **Step 1: Register public and admin taxonomy API routes**

Wire public list routes and admin mutation routes to store methods.

- [ ] **Step 2: Update route inventory**

Add the taxonomy paths to `api_route_inventory`.

- [ ] **Step 3: Render category/tag management panels**

Add `AdminCategoryRow` and `AdminTagRow` to the dashboard model and render compact tables in `/admin`.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no formatting errors.

- [ ] **Step 2: Focused tests**

Run: `cargo test taxonomy_ --test phase1_contract`
Expected: PASS.

- [ ] **Step 3: Full tests and builds**

Run: `cargo test`, `cargo check`, and `cargo leptos build`
Expected: all PASS.

- [ ] **Step 4: IDEA errors**

Use IDEA MCP `get_file_problems(errorsOnly=true)` on changed Rust files and fix error-level findings.
