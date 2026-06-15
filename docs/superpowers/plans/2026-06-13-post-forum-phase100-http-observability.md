# Post Forum Phase 100 HTTP Observability

**Goal:** Add HTTP request/response observability at the Axum SSR entrypoint for PRD 14.3.

## Scope

- Initialize a `tracing-subscriber` formatter at startup.
- Read log filtering from `RUST_LOG` through `EnvFilter`.
- Add `tower_http::trace::TraceLayer` to the SSR router.
- Include request headers in request spans for debugging.
- Log request, response latency, and failure events with microsecond latency units.

## Tasks

- [x] Add RED contract coverage for the SSR observability setup.
- [x] Add `tower-http`, `tracing`, and `tracing-subscriber` SSR dependencies.
- [x] Add `init_tracing`.
- [x] Add `http_trace_layer`.
- [x] Attach the trace layer to the Axum router.
- [x] Verify target test and `cargo check`.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract server_entrypoint_configures_http_observability_layer -- --nocapture`: failed before implementation because `tracing_subscriber::fmt()` setup was missing.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract server_entrypoint_configures_http_observability_layer -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo check --manifest-path post/Cargo.toml`: PASS.

## Context7 Notes

- Library: `/tower-rs/tower-http`.
- `TraceLayer::new_for_http()` is the HTTP-ready trace middleware for Tower/Axum services.
- `DefaultMakeSpan`, `DefaultOnRequest`, `DefaultOnResponse`, and `DefaultOnFailure` configure span creation, request logging, response latency logging, and failure logging.
- `LatencyUnit::Micros` can be used for response and failure latency output.
- `tracing-subscriber` is required to actually emit trace events.

## PRD Coverage

- Supports PRD `14.3` request logging.
- Supports PRD `14.3` error logging through `DefaultOnFailure`.
- Provides a foundation for slow-query and integration-failure logs already emitted by repository and integration layers.
