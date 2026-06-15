# Post Forum Phase 101 README Runtime Accuracy

**Goal:** Keep the project README aligned with the current runtime and final acceptance workflow.

## Scope

- Remove obsolete wording that described the API as only in-memory.
- Document PostgreSQL runtime repositories and SQLx checked macro usage.
- Document the homepage, search, paginated comments, upload, notification, WebSocket, and admin dashboard APIs.
- Document `cargo leptos build`, full test command, and `RUST_LOG` tracing configuration.
- Document `integration_outbox`, RustFS, Elasticsearch, and live external-service test boundaries.

## Tasks

- [x] Add RED README contract coverage.
- [x] Update `post/README.md` to match current runtime behavior.
- [x] Verify README contract test.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract readme_documents_current_runtime_and_verification_workflow -- --nocapture`: failed before the README update because it still contained obsolete in-memory-only wording.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract readme_documents_current_runtime_and_verification_workflow -- --nocapture`: PASS.

## PRD Coverage

- Supports PRD `8` requirement that README explains local startup and configuration.
- Supports PRD `16` final acceptance by documenting Docker Compose dependencies, runtime toggles, validation commands, and core API entry points.
