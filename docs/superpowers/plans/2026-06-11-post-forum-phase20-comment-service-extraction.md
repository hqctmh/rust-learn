# Post Forum Phase 20 Comment Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move pure comment creation and masking rules out of `state.rs` into `services::comments` while preserving comment API behavior.

**Architecture:** `state.rs` remains responsible for locking, finding posts and comments, inserting replies, updating counters, and sending notifications. `services::comments` owns pure rules: content validation/trimming, `CommentNode` construction, deleted-comment masking, and notification body truncation.

**Tech Stack:** Rust, existing comment domain models, existing `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Comment Authoring Rules

**Files:**
- Create/modify: `post/src/services/comments.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::comments::CommentService` and verifies:
- blank content is rejected;
- content is trimmed;
- `author_reply` is true when commenter is the post author;
- deleted comments are masked recursively;
- notification body is truncated to 80 characters.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml comment_service
```

Expected: compile failure because `services::comments` does not exist.

- [x] **Step 3: Implement `CommentService`**

Create methods for building comments, masking deleted comments, and deriving notification body text.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline comment construction, masking, and notification body truncation in `add_comment`, `comments_for_post`, and `delete_own_comment`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
