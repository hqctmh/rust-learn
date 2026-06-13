# Post Forum Phase 77 Admin Role Permission Details

**Goal:** Replace the static admin role "查看权限" button with an expandable permission detail view.

## Scope

- Render role permission details from `role.permissions`.
- Show permission code and name for each permission.
- Keep role update/delete actions unchanged.

## Tasks

- [x] Add RED contract coverage for role permission detail rendering.
- [x] Replace the static permission button with a `details` disclosure.
- [x] Verify admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_expands_role_permission_details -- --nocapture`: failed before implementation with missing `<details class="admin-inline-details">`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_expands_role_permission_details -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 31 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 160 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports RBAC operations by making role permission inspection available directly in the admin role table.
- Removes a static control from the management UI without introducing extra backend complexity.
