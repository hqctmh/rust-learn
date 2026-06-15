# Post Forum Phase 99 Notification Browser WebSocket

**Goal:** Add the browser-side WebSocket notification client required by the PRD realtime push acceptance criteria.

## Scope

- Render a realtime notification status block on the notifications page.
- Connect hydrated browsers to `/ws/notifications/{user_id}` when a session-backed notification center is available.
- Parse incoming WebSocket JSON payloads as `NotificationPush`.
- Surface connected, waiting-for-login, failure, and latest-push states in the page.
- Close the WebSocket during Leptos owner cleanup.

## Tasks

- [x] Add RED page contract coverage for the browser WebSocket client.
- [x] Add hydrate-side `web-sys` features for `Window`, `Location`, `WebSocket`, and `MessageEvent`.
- [x] Add a `NotificationRealtimeClient` component.
- [x] Build the WebSocket URL from the browser location using `ws://` or `wss://`.
- [x] Parse incoming pushes with `serde_json::from_str::<NotificationPush>`.
- [x] Use `Owner::on_cleanup` and `send_wrapper::SendWrapper` to close the socket safely.
- [x] Verify target, notification regression, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract notifications_page_connects_browser_websocket_for_realtime_pushes -- --nocapture`: failed before implementation because `NotificationRealtimeClient` was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract notifications_page_connects_browser_websocket_for_realtime_pushes -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract notification_ -- --nocapture`: PASS, 10 passed.

## Context7 Notes

- Library: `/websites/rs_leptos`.
- Leptos effects run after DOM rendering and generally do not run on the server, making them suitable for browser-specific APIs.
- Leptos cleanup can be registered with `Owner::on_cleanup`.
- The Leptos example wraps non-`Send` browser objects with `send_wrapper::SendWrapper` before storing them in cleanup closures.

## Local API Notes

- `web_sys::WebSocket::new(&str)` returns `Result<WebSocket, JsValue>`.
- `WebSocket::set_onmessage` accepts an optional JS function callback.
- `web_sys::MessageEvent::data()` returns the message payload as `JsValue`.

## PRD Coverage

- Supports PRD `4.8` notification requirement that login sessions can receive realtime WebSocket pushes.
- Supports PRD `14.1` requirement that high-frequency notification events can be delivered asynchronously without blocking page rendering.
- Supports final acceptance criterion: WebSocket can realtime push notifications.
