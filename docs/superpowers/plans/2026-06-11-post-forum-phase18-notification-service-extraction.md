# Post Forum Phase 18 Notification Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move notification push business rules out of `state.rs` into a focused service module without changing the public notification API.

**Architecture:** `state.rs` remains the in-memory adapter and lock owner. `services::notifications` owns the rule for creating a WebSocket push payload from a stored notification only when the recipient has active online connections.

**Tech Stack:** Rust, existing domain models, existing in-memory `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Notification Push Rule

**Files:**
- Create: `post/src/services/mod.rs`
- Create: `post/src/services/notifications.rs`
- Modify: `post/src/lib.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::notifications::NotificationPushService` and verifies:
- online recipients get a `NotificationPush`;
- offline recipients do not get a push;
- generated push payload preserves notification id, recipient id, type, title, and body.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml notification_push_service
```

Expected: compile failure because `services::notifications` does not exist.

- [x] **Step 3: Implement the service module**

Create `NotificationPushService` with a method that takes a push id, active connection count, and `Notification`, then returns `Option<NotificationPush>`.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline push creation in `push_notification` with `NotificationPushService`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
