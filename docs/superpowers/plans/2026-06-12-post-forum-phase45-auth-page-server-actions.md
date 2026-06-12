# Post Forum Phase 45 Auth Page Server Actions

> **For agentic workers:** Continue with `superpowers:test-driven-development` for behavior changes and keep verification evidence fresh.

**Goal:** Turn the login and registration pages from static controls into real Leptos server-action forms backed by `AppState`, so the PRD account entry points can support the publishing and notification workflows.

**Task Status:** Completed and verified on 2026-06-12.

## Scope

This slice wires the public auth pages to runtime auth behavior. It does not yet persist the returned session into a browser-wide auth context; that remains a follow-up for the publish/editor auth flow.

## Tasks

- [x] Add a contract test proving login/register pages submit through Leptos server actions.
- [x] Add `login_user` and `register_user` server functions that delegate to `AppState`.
- [x] Convert `LoginPage` to an `ActionForm` with pending, success, and error feedback.
- [x] Convert `RegisterPage` to an `ActionForm` with password confirmation, pending, success, and error feedback.
- [x] Add compact auth feedback styles that do not disrupt the existing account-page layout.
- [x] Run targeted test, full test, compile check, Leptos build verification, and browser DOM verification.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml auth_pages_submit_through_leptos_server_actions`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 126 passed, 2 ignored.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo leptos build`: PASS.
- In-app browser verification:
  - `/login`: server-action form posts to `/api/login_user...`, fields are `username` and `password`, no horizontal overflow, no console errors.
  - `/register`: server-action form posts to `/api/register_user...`, fields are `username`, `nickname`, `password`, and `confirm_password`, no horizontal overflow, no console errors.
