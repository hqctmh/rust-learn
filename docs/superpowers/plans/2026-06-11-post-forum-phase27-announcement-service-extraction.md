# Post Forum Phase27 Announcement Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move announcement creation, publication, withdrawal, and notification summary rules out of `post/src/state.rs` into a focused announcement service.

**Architecture:** `ForumStore` continues to own admin checks, recipient lookup, notification insertion, and in-memory map writes. `AnnouncementService` owns pure announcement construction and state transitions.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: Announcement Service Rules

**Files:**
- Create: `post/src/services/announcements.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::announcements::AnnouncementService` to build a draft announcement, publish it, withdraw it, and generate a 120-character notification body.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml announcement_service_builds_and_transitions_announcements`

Expected: FAIL because `post::services::announcements` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `AnnouncementService` with `build_draft`, `publish`, `withdraw`, and `notification_body`.

- [x] **Step 4: Wire store methods**

Update `ForumStore::create_announcement`, `publish_announcement`, and `withdraw_announcement` to use `AnnouncementService`, leaving recipient lookup and notification push in `state.rs`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml announcement_service_builds_and_transitions_announcements
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
