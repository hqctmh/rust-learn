# Post Forum Phase 96 Admin Governance Audit Logs

**Goal:** Extend PRD 5.9 audit log coverage to report handling, announcement operations, and taxonomy management.

## Scope

- Add a dedicated `PostgresAdminAuditRepository` for generic admin audit inserts.
- Move content moderation audit writes onto the dedicated repository.
- Record report handling as `report.handle`.
- Record announcement create, update, publish, push, and withdraw actions.
- Record category create, update, and disable actions.
- Record tag create, update, merge, and delete actions.

## Tasks

- [x] Add RED runtime coverage to report handling, announcement, and taxonomy Postgres flows.
- [x] Add `PostgresAdminAuditRepository::insert_audit_log`.
- [x] Wire report, announcement, category, and tag admin operations to audit writes after successful persistence.
- [x] Keep content moderation, user admin, and RBAC audit tests green after moving to the shared audit repository.
- [x] Verify target tests, related audit tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_admin_report_list_and_handle_persist_to_postgres -- --nocapture`: failed before implementation because `report.handle` audit log was missing.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_announcement_admin_and_public_flows_persist_to_postgres -- --nocapture`: failed before implementation because `announcement.create` audit log was missing.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_taxonomy_admin_and_public_flows_persist_to_postgres -- --nocapture`: failed before implementation because `category.create` audit log was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_admin_report_list_and_handle_persist_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_announcement_admin_and_public_flows_persist_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_taxonomy_admin_and_public_flows_persist_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_content_moderation_persists_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_user_admin_persists_to_postgres_and_writes_audit_logs -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_rbac_roles_persist_to_postgres_and_write_audit_logs -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 178 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query!` provides compile-time checked SQL and bind parameter validation.
- `execute` is the documented call style for `INSERT` statements without `RETURNING`.
- `to_jsonb($n::text)` keeps text snapshots compatible with the `jsonb` audit columns while remaining explicit for SQLx.

## PRD Coverage

- Supports `5.9` audit log requirements for report handling, announcement management, and taxonomy management.
- Consolidates generic admin audit writes into a dedicated repository instead of tying them to one business repository.
