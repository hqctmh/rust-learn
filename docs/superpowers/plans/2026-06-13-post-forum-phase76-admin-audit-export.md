# Post Forum Phase 76 Admin Audit Export

**Goal:** Replace the static admin "导出审计日志" button with a real CSV download.

## Scope

- Add `audit_entries_csv` to render audit entries as CSV.
- Escape commas, quotes, and line breaks in exported values.
- Render the admin audit export control as a downloadable `audit-logs.csv` data URI.
- Avoid adding a new dependency for simple percent encoding.

## Tasks

- [x] Add RED behavior coverage for audit CSV escaping.
- [x] Add RED UI contract coverage for the audit CSV download link.
- [x] Implement CSV generation in the admin domain model.
- [x] Replace the static audit export button with a download link.
- [x] Verify admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_audit_log_csv_export_escapes_values -- --nocapture`: failed before implementation with missing `post::domain::admin::audit_entries_csv`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_audit_log_csv_export_escapes_values -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_exports_audit_logs_as_csv_download -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 30 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 159 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports admin audit review by making audit logs exportable from the management UI.
- Keeps admin operations useful without introducing an extra export endpoint for the current table-sized dataset.
