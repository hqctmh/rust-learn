# Post Forum Phase 21 Reaction Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move like/favorite toggle and counter delta rules out of `state.rs` into `services::reactions`.

**Architecture:** `state.rs` remains responsible for locking, validating users and targets, locating posts/comments, and sending notifications. `services::reactions` owns pure rules for toggling a `(user_id, target_id)` pair in a set and applying the resulting counter delta.

**Tech Stack:** Rust, existing reaction domain models, existing in-memory `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Reaction Toggle Rules

**Files:**
- Create/modify: `post/src/services/reactions.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::reactions::ReactionService` and verifies:
- toggling a missing pair inserts it and returns active `true`;
- toggling the same pair removes it and returns active `false`;
- applying active `true` increments a counter;
- applying active `false` decrements a counter but does not go below zero.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml reaction_service
```

Expected: compile failure because `services::reactions` does not exist.

- [x] **Step 3: Implement `ReactionService`**

Create methods for toggling pair membership and applying counter deltas.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline `toggle_set` and `apply_counter_delta` calls with `ReactionService`, then remove the old helper functions if unused.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
