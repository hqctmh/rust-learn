# Post Forum Phase 95 Content Moderation Audit Logs

**Goal:** Record audit logs for admin content moderation operations required by PRD 5.9.

## Scope

- Write audit logs for post moderation actions: take down, restore, delete, pin, unpin, recommend, unrecommend, lock, and unlock.
- Write audit logs for comment moderation actions: delete and recover.
- Keep existing moderation action return values unchanged.
- Store the action, operator, target type, target ID, and after-state snapshot in `audit_logs`.

## Tasks

- [x] Add RED runtime coverage to content moderation Postgres flow requiring audit log rows.
- [x] Add `PostgresModerationRepository::insert_audit_log` with SQLx checked macro usage.
- [x] Wire `AppState` moderation helpers to insert audit logs after successful actions.
- [x] Verify content moderation, existing user/RBAC audit tests, full suite, check, Leptos build, and diff check.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_content_moderation_persists_to_postgres -- --nocapture`: failed before implementation because `post.take_down` audit log was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_content_moderation_persists_to_postgres -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_user_admin_persists_to_postgres_and_writes_audit_logs -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_rbac_roles_persist_to_postgres_and_write_audit_logs -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_audit -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 178 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## Context7 Notes

- Library: `/websites/rs_sqlx`.
- `query!` provides compile-time checked SQL with bind parameter validation.
- `execute` is the documented call style for `INSERT` statements without `RETURNING`.
- Explicit `to_jsonb($6::text)` casts keep audit snapshot inserts type-stable for SQLx.

## PRD Coverage

- Supports `5.9` audit log requirement for admin key operations.
- Extends audit coverage beyond user and RBAC administration into content moderation.
