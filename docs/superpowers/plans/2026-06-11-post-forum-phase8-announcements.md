# Post Forum Announcement Management Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add announcement management, publishing, withdrawal, read state, and notification fanout so PRD 5.7 is supported by real system behavior.

**Architecture:** Keep the current in-memory `ForumStore` pattern. A focused `announcements` domain module owns request/response contracts; `ForumStore` validates admin permissions, stores announcement state, projects published announcements into the homepage, and pushes announcement notifications to target users.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, time, existing contract tests.

---

### Task 1: Announcement Domain and Contract Tests

**Files:**
- Create: `post/src/domain/announcements.rs`
- Modify: `post/src/domain/mod.rs`
- Modify: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Write failing announcement workflow test**

Add `announcement_contract_supports_publish_push_withdraw_and_read_state` asserting:
- non-admin users cannot create announcements
- admin can create a draft announcement
- publishing changes status to `Published`
- publishing pushes `NotificationType::Announcement` to target users
- homepage announcements include the published announcement
- users can mark the announcement read
- withdrawal removes it from public announcement listing

- [ ] **Step 2: Write failing route inventory test**

Extend route tests for:
- `GET /api/announcements`
- `POST /api/announcements/{announcement_id}/read`
- `GET /api/admin/announcements`
- `POST /api/admin/announcements`
- `POST /api/admin/announcements/{announcement_id}/publish`
- `POST /api/admin/announcements/{announcement_id}/withdraw`

- [ ] **Step 3: Implement announcement types**

Create `AnnouncementStatus`, `AnnouncementAudience`, `CreateAnnouncementRequest`, `AnnouncementItem`, and `AnnouncementReadState` with serde derives and validation methods.

### Task 2: Store Behavior

**Files:**
- Modify: `post/src/state.rs`

- [ ] **Step 1: Extend store state**

Add `announcements: HashMap<Uuid, AnnouncementItem>` and `announcement_reads: HashSet<(Uuid, Uuid)>` to `ForumData`.

- [ ] **Step 2: Implement admin operations**

Add `create_announcement`, `list_admin_announcements`, `publish_announcement`, and `withdraw_announcement`, all guarded by admin permission.

- [ ] **Step 3: Implement public operations**

Add `public_announcements` and `mark_announcement_read`; public list returns only published, currently effective, unexpired announcements.

- [ ] **Step 4: Project published announcements into homepage**

When the store has published announcements, `home_page` replaces the static homepage announcement module with published announcement titles and date labels.

### Task 3: API and Admin UI

**Files:**
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/domain/admin.rs`
- Modify: `post/src/pages/admin.rs`

- [ ] **Step 1: Register API routes**

Wire the announcement routes to store methods with JSON request/response bodies.

- [ ] **Step 2: Update route inventory**

Add all announcement paths to `api_route_inventory`.

- [ ] **Step 3: Add admin dashboard announcement rows**

Add `AdminAnnouncementRow` and render an "公告推送" management table in `/admin`.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no formatting errors.

- [ ] **Step 2: Focused tests**

Run: `cargo test announcement_ --test phase1_contract`
Expected: PASS.

- [ ] **Step 3: Full tests and builds**

Run: `cargo test`, `cargo check`, and `cargo leptos build`
Expected: all PASS.

- [ ] **Step 4: IDEA errors**

Use IDEA MCP `get_file_problems(errorsOnly=true)` on changed Rust files and fix error-level findings.
