# Post Forum Phase 71 Admin Role Actions

**Goal:** Make the admin role management panel create, update, and delete roles through session-backed Leptos actions.

## Scope

- Add role create/update/delete server functions in `page_data`.
- Wire the role management panel's create form to `CreateAdminRole`.
- Wire each role row's update and delete controls to `UpdateAdminRole` and `DeleteAdminRole`.
- Refresh the admin dashboard from action results and show success/failure feedback.
- Keep the permission input simple as comma-separated permission codes.

## Tasks

- [x] Add RED contract coverage for role create/update/delete server actions and forms.
- [x] Implement `create_admin_role(session_id, code, name, permission_codes)`.
- [x] Implement `update_admin_role(session_id, role_code, name, permission_codes)`.
- [x] Implement `delete_admin_role(session_id, role_code)`.
- [x] Convert comma-separated permission codes into `Vec<String>` server-side.
- [x] Render create, update, and delete `ActionForm`s on the admin page.
- [x] Verify admin page tests, RBAC tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_manages_roles_with_session_actions -- --nocapture`: failed before implementation with `admin role server action missing fragment: pub async fn create_admin_role(`.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_manages_roles_with_session_actions -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_ -- --nocapture`: PASS, 12 passed.
  - `cargo test --manifest-path post/Cargo.toml rbac_ -- --nocapture`: PASS, 5 passed.
  - `cargo test --manifest-path post/Cargo.toml`: PASS, 152 passed, 2 ignored.
  - `cargo check --manifest-path post/Cargo.toml`: PASS.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports `5.2` RBAC management: create, update, and delete roles from the management UI.
- Supports `13` permission rules by routing role changes through session-backed server functions and existing RBAC backend validation.
