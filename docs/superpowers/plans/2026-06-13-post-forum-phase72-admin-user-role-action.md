# Post Forum Phase 72 Admin User Role Action

**Goal:** Make the admin user table's "调整角色" control submit a real session-backed role assignment action.

## Scope

- Add `update_admin_user_roles(session_id, target_user_id, roles)`.
- Convert comma-separated role codes into `Vec<String>` server-side.
- Render each user row's role assignment control as an `ActionForm`.
- Refresh the admin dashboard from the action result and show user-role success/failure feedback.

## Tasks

- [x] Add RED contract coverage for user role assignment server action and UI form.
- [x] Implement `update_admin_user_roles` through `AppState::update_user_roles`.
- [x] Render a role-code input with the user's current roles as its value.
- [x] Keep existing user enable/disable actions working.
- [x] Verify admin page, user admin, RBAC, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_user_roles_with_session_action -- --nocapture`: failed before implementation with `admin user role server action missing fragment: pub async fn update_admin_user_roles(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_updates_user_roles_with_session_action -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 13 passed.
  - `cargo test --manifest-path post/Cargo.toml user_admin -- --nocapture`: PASS, 4 passed.
  - `cargo test --manifest-path post/Cargo.toml rbac_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 153 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `5.3` user management: admins can modify user roles from the management UI.
- Supports `5.2` RBAC management and `13` permission rules by routing role assignments through the session-backed RBAC/user-admin backend.
