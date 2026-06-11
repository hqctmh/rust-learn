# Post Forum Phase 14 Comment Interactions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Complete the user-side comment delete, comment like, and comment report capabilities required by `post/prd.md` sections 4.3, 4.4, 4.8, 4.11, 12, 13, and 16.

**Architecture:** Keep comment behavior in `ForumStore` so API handlers stay thin. Reuse existing `CommentNode`, `ToggleResult`, `ReportItem`, notification, and masking patterns; add a comment-like relation set to `ForumData`; expose explicit comment routes under `/api/comments/{comment_id}/...`; surface actions on the post detail page.

**Tech Stack:** Rust, Leptos, Axum 0.8 routing and extractors, serde, uuid, existing `ForumError` and in-memory store patterns.

---

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write failing behavior test**

Add `comment_interaction_contract_supports_author_delete_like_and_report` proving:
- a comment author can delete their own comment;
- another normal user cannot delete that comment;
- deleted comments remain visible as `该评论已被删除`;
- deleting a comment decrements the post comment count;
- liking a comment toggles on and off and updates `like_count`;
- liking a comment notifies the comment author;
- creating a comment report records `ReportTargetType::Comment`, target title, reporter, and pending status.

- [x] **Step 2: Write failing route test**

Add `comment_interaction_routes_are_registered` asserting:
- `/api/comments/{comment_id}/delete`
- `/api/comments/{comment_id}/like`
- `/api/comments/{comment_id}/report`

- [x] **Step 3: Run targeted tests and verify red**

Run:

```bash
cargo test comment_interaction --test phase1_contract
```

Expected: fail because store methods and route inventory do not exist.

### Task 2: Store Behavior

**Files:**
- Modify: `post/src/state.rs`

- [x] **Step 1: Add comment-like relation set**

Add `liked_comments: HashSet<(Uuid, Uuid)>` to `ForumData` and seed initialization.

- [x] **Step 2: Implement user-side methods**

Add:
- `ForumStore::delete_own_comment(user_id, comment_id)`
- `ForumStore::toggle_comment_like(user_id, comment_id)`
- `ForumStore::report_comment(user_id, comment_id, request)`

Rules:
- missing user returns unauthorized;
- non-author delete returns forbidden;
- deleted comments render as `该评论已被删除`;
- post comment count stays consistent;
- comment like count toggles between 0 and 1 for the same user/comment;
- liking someone else's comment emits `NotificationType::CommentLiked`;
- report comment delegates to the existing report data model with `ReportTargetType::Comment`.

- [x] **Step 3: Run targeted test and verify green**

Run:

```bash
cargo test comment_interaction_contract --test phase1_contract
```

Expected: pass.

### Task 3: API and UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/pages/post_detail.rs`

- [x] **Step 1: Add API handlers and routes**

Add:
- `POST /api/comments/{comment_id}/delete`
- `POST /api/comments/{comment_id}/like`
- `POST /api/comments/{comment_id}/report`

Use `UserActionRequest` for delete/like and `CreateReportRequest` for report, following the existing MVP demo-user fallback pattern.

- [x] **Step 2: Add route inventory entries**

Add the three comment routes to `api_route_inventory()`.

- [x] **Step 3: Surface detail-page actions**

Ensure post detail comments expose visible action controls for:
- 点赞评论
- 删除自己的评论
- 举报评论

- [x] **Step 4: Run route test and verify green**

Run:

```bash
cargo test comment_interaction_routes_are_registered --test phase1_contract
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
- `post/src/state.rs`
- `post/src/api.rs`
- `post/src/app.rs`
- `post/src/pages/post_detail.rs`
- `post/tests/phase1_contract.rs`

Expected: no errors.
