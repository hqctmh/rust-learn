# Post Forum Phase 63 Admin Report Handling Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Make the admin report-handling table's "标记已处理 / 驳回" controls submit real session-backed actions instead of remaining static buttons.

**Task Status:** Completed and verified on 2026-06-13.

## Scope

This slice wires the management page to the existing report handling backend. The dashboard report row now carries the target `report_id`, and each pending report row submits Leptos `ActionForm` controls for marking a report handled or rejected. After each mutation, the server function reloads the admin dashboard so report status, available actions, and governance counts reflect the latest backend state.

## Tasks

- [x] Add a RED contract test for admin report handling action wiring.
- [x] Expose `report_id` on `domain::admin::AdminReportRow`.
- [x] Map PostgreSQL/in-memory report rows into dashboard report rows with target IDs.
- [x] Add `handle_admin_report(session_id, report_id)` server function.
- [x] Add `reject_admin_report(session_id, report_id)` server function.
- [x] Replace static report handling buttons with `ActionForm` controls.
- [x] Submit hidden `session_id` and `report_id` fields.
- [x] Disable controls while pending or when no session is available.
- [x] Render success and failure feedback after report handling changes.

## Verification Evidence

- RED:
  - `cargo test --manifest-path post/Cargo.toml admin_page_handles_reports_with_session_actions -- --nocapture`: failed before implementation with `admin dashboard report rows should expose target report id fragment: pub report_id: Uuid,`.
- GREEN:
  - `cargo test --manifest-path post/Cargo.toml admin_page_handles_reports_with_session_actions -- --nocapture`: PASS.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 144 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Covers the PRD admin requirement to process user reports.
- Covers the PRD admin requirement to reject invalid reports.
- Keeps report handling session-backed, so only an authenticated admin session can change report status.
