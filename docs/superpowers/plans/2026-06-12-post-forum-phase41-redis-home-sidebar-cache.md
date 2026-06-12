# Post Forum Phase 41 Redis Home Sidebar Cache

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:test-driven-development` for implementation and `superpowers:verification-before-completion` before reporting completion.

**Goal:** Add a Redis-backed homepage sidebar cache boundary so the Dense Workbench homepage can serve category statistics, hot tags, announcements, and active authors without querying every sidebar module on every request.

**Architecture:** Keep PostgreSQL as the authoritative source. Add `HomeSidebarSnapshot` as the serializable cache payload for the four homepage sidebar modules, and add `RedisHomeCacheRepository` as the Redis adapter. `AppState::home_page` keeps loading the topic list from PostgreSQL, then optionally reads/writes the sidebar cache when `HOME_SIDEBAR_CACHE_ENABLED=true`. Redis failures are treated as cache misses and do not block homepage rendering.

**Tech Stack:** Rust 2024, Redis `1.2.2` async Tokio client, Serde JSON, existing homepage domain DTOs.

**Task Status:** Completed and verified on 2026-06-12.

---

## Scope

This slice covers homepage sidebar cache read/write boundaries. It does not introduce view-count writeback, hot-topic caching, or a scheduled statistics materializer.

## Tasks

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [x] Add `redis_home_sidebar_cache_contract_preserves_all_sidebar_modules`.
- [x] Add `app_state_home_runtime_has_redis_sidebar_cache_boundary`.
- [x] Verify the cache snapshot contains categories, hot tags, announcements, and active authors.
- [x] Verify runtime config exposes `HOME_SIDEBAR_CACHE_ENABLED` and `HOME_SIDEBAR_CACHE_TTL_SECONDS`.

### Task 2: Redis Cache Repository

**Files:**
- Modify: `post/src/repositories/home.rs`

- [x] Add `HomeSidebarSnapshot`.
- [x] Add JSON encode/decode helpers.
- [x] Add `RedisHomeCacheRepository::from_url`.
- [x] Add `try_read_sidebar` using Redis `GET`.
- [x] Add `write_sidebar` using Redis `SET ... EX ...`.
- [x] Use stable cache key `home:sidebar:v1`.

### Task 3: AppState Runtime Integration

**Files:**
- Modify: `post/src/state.rs`
- Modify: `post/tests/integration_live.rs`

- [x] Add runtime config for `home_sidebar_cache_enabled`.
- [x] Add runtime config for `home_sidebar_cache_ttl_seconds`.
- [x] Read sidebar cache after loading homepage topics.
- [x] On cache miss, load sidebar modules from PostgreSQL and write snapshot back to Redis.
- [x] Keep default cache disabled so normal local tests do not require Redis.

## Verification Evidence

- `cargo test --manifest-path post/Cargo.toml redis_home_sidebar_cache_contract_preserves_all_sidebar_modules`: PASS, 1 passed.
- `cargo test --manifest-path post/Cargo.toml app_state_home_runtime_has_redis_sidebar_cache_boundary`: PASS, 1 passed.
- `cargo check --manifest-path post/Cargo.toml`: PASS.
- `cargo test --manifest-path post/Cargo.toml`: PASS, 114 passed, 1 ignored.
- `cargo leptos build`: PASS.

## PRD Coverage

- Covers the homepage performance requirement that Redis can cache hot tags, category statistics, active authors, and other sidebar data.
- Covers the homepage resilience requirement that sidebar cache failures must not make the main topic list unavailable.
- Aligns with existing outbox cache invalidation keys such as `home:sidebar:*`.
