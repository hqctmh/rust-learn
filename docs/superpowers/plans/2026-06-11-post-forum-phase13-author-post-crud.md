# Post Forum Phase 13 Author Post CRUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Complete the author-side post editing, own-post deletion, and draft autosave capabilities required by `post/prd.md` sections 4.2, 13, and 16.

**Architecture:** Keep the existing in-memory `ForumStore` as the contract boundary for this MVP. Add request/response structs in `domain::posts`, implement ownership checks in `ForumStore`, expose explicit author CRUD routes from `api.rs`, and register an edit route in the Leptos app so the UI surface matches the backend capability.

**Tech Stack:** Rust, Leptos, Axum, serde, uuid, time, existing `ForumError` and `ForumStore` patterns.

---

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing tests**

Add a test proving:
- logged-in users can autosave a draft;
- drafts do not appear in public post lists;
- the author can publish/update the draft;
- a different user cannot edit or delete the post;
- the author can delete their own post;
- deleted posts disappear from public lists.

Add route inventory assertions for:
- `/posts/sample/edit`
- `/api/posts/drafts/autosave`
- `/api/posts/{post_id}/update`
- `/api/posts/{post_id}/delete`

- [x] **Step 2: Run targeted tests and verify red**

Run:

```bash
cargo test author_post --test phase1_contract
```

Expected: fail because the new store methods and routes do not exist yet.

### Task 2: Domain and Store

**Files:**
- Modify: `post/src/domain/posts.rs`
- Modify: `post/src/state.rs`

- [x] **Step 1: Add request structs**

Add:
- `UpdatePostRequest`
- `AutosaveDraftRequest`

The fields should match the existing `CreatePostRequest` shape and include an optional `post_id` for autosave updates.

- [x] **Step 2: Implement ownership-checked store methods**

Add:
- `ForumStore::autosave_draft(author_id, request)`
- `ForumStore::update_post(author_id, post_id, request)`
- `ForumStore::delete_own_post(author_id, post_id)`

The methods must reject missing users, disabled users, missing posts, and non-owner edits/deletes. Published posts should remain public after update; deleted posts should be hidden from `list_posts()` and `home_page()`.

- [x] **Step 3: Run targeted tests and verify green**

Run:

```bash
cargo test author_post --test phase1_contract
```

Expected: pass.

### Task 3: API and App Routes

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/pages/editor.rs`

- [x] **Step 1: Add API routes**

Add:
- `POST /api/posts/drafts/autosave`
- `POST /api/posts/{post_id}/update`
- `POST /api/posts/{post_id}/delete`

Use the existing demo-user query pattern for MVP user identity unless the request already carries a user id.

- [x] **Step 2: Add edit route and UI affordance**

Register `/posts/:id/edit` to reuse `EditorPage`, add `/posts/sample/edit` to `primary_routes()`, and update the editor copy/actions so editing and autosave are first-class visible states.

- [x] **Step 3: Run route tests**

Run:

```bash
cargo test author_post_routes_are_registered --test phase1_contract
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
- `post/src/domain/posts.rs`
- `post/src/state.rs`
- `post/src/api.rs`
- `post/src/app.rs`
- `post/src/pages/editor.rs`
- `post/tests/phase1_contract.rs`

Expected: no errors.
