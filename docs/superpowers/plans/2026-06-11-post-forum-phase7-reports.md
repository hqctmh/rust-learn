# Post Forum Report Moderation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a working report submission and admin moderation slice that satisfies PRD 4.11 and 5.8.

**Architecture:** Keep the current in-memory `ForumStore` pattern and expose stable Axum JSON APIs. Reports live in a focused domain module and are projected into the existing admin dashboard so the UI, API, tests, and PRD inventory all agree.

**Tech Stack:** Rust, Leptos, Axum JSON routes, serde, uuid, time, existing contract tests.

---

### Task 1: Report Domain Contract

**Files:**
- Create: `post/src/domain/reports.rs`
- Modify: `post/src/domain/mod.rs`
- Test: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add a failing contract test**

Add `report_contract_supports_submission_and_admin_resolution` asserting that a logged-in user can report a post, admins can list reports, mark one handled, reject another, and receive audit metadata.

- [ ] **Step 2: Run the focused test**

Run: `cargo test report_contract --test phase1_contract`
Expected: FAIL because `post::domain::reports` and store methods do not exist.

- [ ] **Step 3: Implement report types**

Create `ReportTargetType`, `ReportStatus`, `CreateReportRequest`, `HandleReportRequest`, and `ReportItem` with serde derives and validation helpers. Supported targets are `post`, `comment`, and `user`; statuses are pending, handled, and rejected.

- [ ] **Step 4: Export the module**

Add `pub mod reports;` to `post/src/domain/mod.rs`.

### Task 2: Store and API Wiring

**Files:**
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`
- Test: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Extend state**

Add `reports: HashMap<Uuid, ReportItem>` to `ForumData`, initialize it in `seeded`, and implement `create_report`, `list_reports`, and `handle_report`.

- [ ] **Step 2: Enforce permissions**

`create_report` validates target existence and reporter identity. `list_reports` and `handle_report` require an admin user; non-admin users return `Forbidden`.

- [ ] **Step 3: Add API routes**

Register `POST /api/reports`, `GET /api/admin/reports`, and `POST /api/admin/reports/{report_id}/handle`.

- [ ] **Step 4: Update route inventory**

Expose the new routes through `post::app::api_route_inventory()`.

### Task 3: Admin UI Projection

**Files:**
- Modify: `post/src/domain/admin.rs`
- Modify: `post/src/pages/admin.rs`
- Test: `post/tests/phase1_contract.rs`

- [ ] **Step 1: Add report rows to dashboard**

Add `AdminReportRow` and `reports: Vec<AdminReportRow>` to `AdminDashboard`. Seed demo rows that show pending report type, reason, reporter, status, and actions.

- [ ] **Step 2: Render report handling panel**

Add a `举报处理` table to admin page with actions `标记已处理`, `驳回`, and `删除违规内容`.

- [ ] **Step 3: Update inventory test**

Ensure `dense_workbench_ui_exposes_prd_system_features` continues to include `举报处理` and route tests include the new API paths.

### Task 4: Verification

**Files:**
- All changed files

- [ ] **Step 1: Format**

Run: `cargo fmt`
Expected: no formatting diff required afterward.

- [ ] **Step 2: Focused tests**

Run: `cargo test report_contract --test phase1_contract`
Expected: PASS.

- [ ] **Step 3: Full tests**

Run: `cargo test`
Expected: PASS.

- [ ] **Step 4: Build check**

Run: `cargo check`
Expected: PASS.

- [ ] **Step 5: IDEA problem check**

Use IDEA MCP problem inspection only if Rust file problem tooling is available; fix error-level findings.
