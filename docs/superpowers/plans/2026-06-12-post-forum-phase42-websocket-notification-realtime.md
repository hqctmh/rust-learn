# Post Forum Phase 42 WebSocket Notification Realtime Push

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:test-driven-development` for behavior changes and `superpowers:verification-before-completion` before reporting completion.

**Goal:** Upgrade the notification WebSocket path from a connection-time pending push drain to an active connection loop that keeps delivering new pending notifications while the socket remains open.

**Architecture:** Keep `AppState` and `ForumStore` as the current notification state owner. `NotificationPushService` now owns the WebSocket JSON payload and ack parsing rules. The Axum WebSocket session sends existing pending pushes immediately, then uses `tokio::select!` to handle client ack messages and periodically poll `pending_notification_pushes` for new pushes. Sent pushes are acknowledged server-side after successful delivery to avoid duplicate sends.

**Tech Stack:** Rust 2024, Axum 0.8 WebSocketUpgrade, Tokio interval/select, Serde JSON, existing notification domain DTOs.

**Task Status:** Completed and verified on 2026-06-12.

---

## Scope

This slice implements server-side real-time delivery for pending notification pushes over the existing WebSocket route. It does not yet add a browser-side WebSocket client component or a NATS notification consumer.

## Tasks

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] Add `notification_push_service_builds_websocket_json_payload_and_ack_message`.
- [x] Add `notification_websocket_session_polls_pending_pushes_during_connection`.
- [x] Verify JSON payload includes push ID, notification ID, title, body, and message type.
- [x] Verify ack parser accepts both structured JSON ack messages and raw UUID fallback.
- [x] Verify API source contains interval/select pending-push loop.

### Task 2: Notification Push Service Rules

**Files:**
- Modify: `post/src/services/notifications.rs`

- [x] Add `NotificationPushService::websocket_payload`.
- [x] Add `NotificationPushService::ack_message_to_push_id`.
- [x] Keep `build_pending_push` as the online-connection gate for business notifications.

### Task 3: Axum WebSocket Session Loop

**Files:**
- Modify: `post/src/api.rs`

- [x] Keep `/ws/notifications/{user_id}` on `WebSocketUpgrade`.
- [x] Send existing pending pushes immediately after connect.
- [x] Use `tokio::time::interval` and `tokio::select!` to send new pending pushes while connected.
- [x] Parse incoming ack messages through the service boundary.
- [x] Disconnect socket state on close or send failure.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml notification_push_service_builds_websocket_json_payload_and_ack_message`: PASS, 1 passed.
- `cargo test --manifest-path post/Cargo.toml notification_websocket_session_polls_pending_pushes_during_connection`: PASS, 1 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 116 passed, 1 ignored.
- `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD requirement that logged-in users establish a WebSocket connection.
- Covers the PRD requirement that new notifications are pushed through WebSocket while the user is online.
- Keeps historical notification APIs and pending-push acknowledgement paths available for reconnect and fallback flows.
