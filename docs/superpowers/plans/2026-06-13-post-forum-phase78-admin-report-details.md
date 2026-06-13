# Post Forum Phase 78 Admin Report Details

**Goal:** Replace the static admin report "查看详情" button with an expandable report detail view.

## Scope

- Render report target, type, reason, reporter, and status in a row-level detail disclosure.
- Keep handle/reject server actions unchanged.

## Tasks

- [x] Add RED contract coverage for report detail rendering.
- [x] Replace the static report detail button with a `details` disclosure.
- [x] Verify admin tests, full suite, check, and Leptos build.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_expands_report_details -- --nocapture`: failed before implementation with missing report detail summary markup.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_page_expands_report_details -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract admin_ -- --nocapture`: PASS, 32 passed.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml`: PASS, 161 passed, 2 ignored.
  - `cargo leptos build`: PASS.

## PRD Coverage

- Supports report handling by letting admins inspect the report context inline before resolving or rejecting.
- Removes the last explicit static detail button from the report management table.
