# Post Forum Phase 22 Follow Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move follow/unfollow relationship rules out of `state.rs` into `services::follows`.

**Architecture:** `state.rs` remains responsible for locking and checking whether users exist. `services::follows` owns pure rules for rejecting self-follow and toggling `(follower_id, followee_id)` membership while returning `FollowState`.

**Tech Stack:** Rust, existing `FollowState` domain model, existing in-memory `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Follow Toggle Rules

**Files:**
- Create/modify: `post/src/services/follows.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::follows::FollowService` and verifies:
- following yourself returns the existing conflict error;
- first toggle inserts the relationship and returns `following = true`;
- second toggle removes the relationship and returns `following = false`.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml follow_service
```

Expected: compile failure because `services::follows` does not exist.

- [x] **Step 3: Implement `FollowService`**

Create a method that validates `follower_id != followee_id`, toggles set membership, and returns `FollowState`.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline self-follow and toggle logic in `ForumStore::follow_user`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
