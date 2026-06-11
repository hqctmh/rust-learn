# Post Forum Phase 17 Notification Push Service Boundary Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the notification push contract required by the PRD while establishing the first service boundary that keeps new business behavior out of `state.rs` where possible.

**Architecture:** Keep `ForumStore` as the current in-memory adapter, but move notification push state modeling into `domain::notifications` and expose narrow store methods for socket connect/disconnect, pending pushes, and acknowledgements. API handlers only adapt HTTP/WebSocket routes to store calls.

**Tech Stack:** Rust, Leptos SSR, Axum 0.8, serde, uuid, existing in-memory `ForumStore`.

---

### Task 1: Notification Push Contract

**Files:**
- Modify: `post/tests/phase1_contract.rs`
- Modify: `post/src/domain/notifications.rs`
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/pages/notifications.rs`

- [x] **Step 1: Write the failing tests**

Add tests that require:
- `/ws/notifications/{user_id}` route inventory.
- `/api/notifications/online`, `/api/notifications/pending-pushes`, and `/api/notifications/pending-pushes/{push_id}/ack` route inventory.
- A connected user receives a pending push payload when a business notification is created.
- Acknowledging the push removes it.
- Disconnecting the user reduces online connection count.

- [x] **Step 2: Run the tests to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml notification_push
```

Expected: failure because push types and store methods are not implemented yet.

- [x] **Step 3: Implement the domain contract**

Add `NotificationPush`, `NotificationPushAckRequest`, and `NotificationConnectionStats` to `post/src/domain/notifications.rs`.

- [x] **Step 4: Implement the in-memory adapter**

Extend `ForumData` with online connection counts and pending push queues. Update the internal notification writer so connected recipients receive pending push payloads.

- [x] **Step 5: Register API and WebSocket routes**

Add HTTP route inventory and handlers for online stats, pending pushes, push acknowledgement, and the WebSocket route contract.

- [x] **Step 6: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cargo leptos build --manifest-path post/Cargo.toml
```

Expected: all commands exit 0.
