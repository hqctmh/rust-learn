# Post Forum Content Moderation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add real post and comment moderation operations for PRD 5.4 and 5.5, including takedown, restore, delete, pin/unpin, comment delete/recover, and admin list APIs.

**Architecture:** Reuse existing `PostStatus` and `CommentNode.deleted` fields. Add a focused `moderation` domain module for admin list rows and mutation results; persist pin state in `ForumStore`; keep public list/detail behavior separated from admin operations.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, existing contract tests.

---

### Task 1: Moderation Domain and Tests

**Files:**
- Create: `post/src/domain/moderation.rs`
- Modify: `post/src/domain/mod.rs`
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add failing workflow test**

Add `content_moderation_contract_supports_post_and_comment_actions` asserting:
- non-admin users cannot list moderation posts
- admin can list moderation posts
- admin can take down a post and public `list_posts` excludes it
- admin can restore the post and public `list_posts` includes it
- admin can pin and unpin a post
- admin can soft-delete a comment, public comments show `deleted=true` and content placeholder
- admin can recover the comment and comment count is restored
- admin can permanently mark a post deleted and public list excludes it

- [ ] **Step 2: Add failing route test**

Add `content_moderation_routes_are_registered` covering admin post and comment moderation routes.

- [ ] **Step 3: Implement moderation row types**

Create `ModerationPostRow`, `ModerationCommentRow`, `ModerationPostAction`, and `ModerationCommentAction`.

### Task 2: Store Behavior

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Add pin state**

Add `pinned_posts: HashSet<Uuid>` to `ForumData`.

- [ ] **Step 2: Implement post moderation methods**

Add `admin_posts`, `take_down_post`, `restore_post`, `delete_post`, `pin_post`, and `unpin_post`, all guarded by admin permission.

- [ ] **Step 3: Implement comment moderation methods**

Add `admin_comments`, `delete_comment`, and `recover_comment`, all guarded by admin permission. Deleted public comments must retain a placeholder text `该评论已被删除`.

### Task 3: API and Admin UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/domain/admin.rs`
- Modify: `post/src/pages/admin.rs`

- [ ] **Step 1: Register moderation API routes**

Wire admin post/comment list and action routes to store methods.

- [ ] **Step 2: Update route inventory**

Add moderation route paths to `api_route_inventory`.

- [ ] **Step 3: Update admin page actions**

Ensure the existing post/comment management panels show actions for takedown, restore, delete, pin, unpin, comment delete, and comment recover.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no formatting errors.

- [ ] **Step 2: Focused tests**

Run: `cargo test content_moderation --test phase1_contract`
Expected: PASS.

- [ ] **Step 3: Full tests and builds**

Run: `cargo test`, `cargo check`, and `cargo leptos build`
Expected: all PASS.

- [ ] **Step 4: IDEA errors**

Use IDEA MCP `get_file_problems(errorsOnly=true)` on changed Rust files and fix error-level findings.
