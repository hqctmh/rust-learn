# Post Forum Phase 3 Notifications Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable notification center foundation for PRD 4.8: business actions create notifications, users can list history, see unread counts, mark one notification read, and mark all read.

**Architecture:** Extend the existing `domain::notifications` model with a `NotificationCenter` aggregate and demo feed. Store notifications in `ForumStore` as recipient-keyed vectors. Wire notification creation into comments, likes, and followed-user publishing, then expose `GET /api/notifications`, `POST /api/notifications/{id}/read`, and `POST /api/notifications/read-all`.

**Tech Stack:** Rust 2024, Axum 0.8, Leptos 0.8, Serde, existing in-memory `ForumStore`, existing contract tests.

**Task Status:** Completed and verified on 2026-06-12.

---

## Scope

This slice implements durable in-process notification behavior and UI/API contracts. It does not implement WebSocket transport or NATS consumers yet; those can be added later behind the same store/API behavior.

## Tasks

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] Add tests that assert:
  - Commenting on another user's post creates a `PostCommented` notification for the post author.
  - Liking another user's post creates a `PostLiked` notification for the post author only when the like becomes active.
  - A follower receives `FollowedUserPosted` when a followed user publishes a post.
  - `mark_notification_read` sets `read_at`.
  - `mark_all_notifications_read` clears unread count.
  - `/notifications` and notification API routes are registered.

### Task 2: Domain Model

**Files:**
- Modify: `post/src/domain/notifications.rs`

- [x] Add `NotificationCenter`, `NotificationReadRequest`, `UnreadCount`.
- [x] Add `notification_demo_center()` for UI rendering before WebSocket integration.

### Task 3: Store and API

**Files:**
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`

- [x] Store notifications by recipient.
- [x] Generate notifications from comment, like, and followed-user post events.
- [x] Add list, mark-one-read, mark-all-read store methods.
- [x] Add notification API routes and inventory entries.

### Task 4: Notification Page

**Files:**
- Create: `post/src/pages/notifications.rs`
- Modify: `post/src/pages/mod.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/components/mod.rs`
- Modify: `post/style/main.css`

- [x] Add `/notifications` route.
- [x] Link top-nav notification button to `/notifications`.
- [x] Render compact notification center with unread count and type labels.

### Task 5: Verification

- [x] Run `cargo fmt`.
- [x] Run `cargo test`.
- [x] Run `cargo check`.
- [x] Run `cargo leptos build`.
- [x] IDEA error check not required by project instruction.
- [x] Browser/API verify `/api/notifications` and `/notifications`.

## Self-Review

- Covers PRD 4.8 history, unread, single read, all read, persisted notification concept, and business-event creation.
- Leaves WebSocket/NATS as transport/integration follow-up, not a blocker for current runnable behavior.

## Current verification evidence (2026-06-12)

- `cargo test --manifest-path post/Cargo.toml`: PASS, 103 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
- Browser verification at `/notifications?verify=after-fix`: PASS; page shows notification center, unread count, history list, `scrollWidth=1280`, `clientWidth=1280`, and no new console errors after navigation.
- Regression coverage added for wasm-safe notification fallback data, preventing `OffsetDateTime::now_utc()` during hydration fallback rendering.
