# Post Forum Phase29 Moderation Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move content moderation state-transition rules out of `post/src/state.rs` into a focused moderation service.

**Architecture:** `ForumStore` continues to own admin checks, post/comment lookup, in-memory map writes, and pinned-post storage. `ModerationService` owns pure transformations for post status actions, post pin actions, comment delete/recover effects, and moderation row projections.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: Moderation Service Rules

**Files:**
- Create: `post/src/services/moderation.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::moderation::ModerationService` to apply post status changes, reject pinning deleted posts, compute comment delete/recover count deltas, and build moderation rows.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml moderation_service_applies_post_and_comment_actions`

Expected: FAIL because `post::services::moderation` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `ModerationService` with post status action, pin action, comment deleted action, comment count application, post row projection, and comment row flattening.

- [x] **Step 4: Wire store methods**

Update `ForumStore::admin_posts`, `admin_comments`, `set_post_status`, `set_post_pin`, and `set_comment_deleted` to use `ModerationService`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml moderation_service_applies_post_and_comment_actions
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
