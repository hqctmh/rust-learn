# Post Forum Phase 2 Search Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable search foundation that satisfies PRD 4.9 and the homepage search entry: `/api/search` plus `/search?q=...` result page with keyword, filter, sort, and highlight support.

**Architecture:** Create a `domain::search` module for query/result DTOs and deterministic in-memory search over the same Dense Workbench seed used by the homepage. Add `ForumStore::search`, expose it through Axum `GET /api/search`, then add a Leptos search results page and wire the top-nav search form to `/search`.

**Tech Stack:** Rust 2024, Leptos 0.8 Router, Axum 0.8 `Query<T>`, Serde, existing `ForumStore`, existing contract tests.

---

## Scope

This slice does not integrate Elasticsearch yet. It establishes the product and API contract so the later Elasticsearch-backed implementation can replace the in-memory matcher without changing the page or API shape.

## Tasks

### Task 1: Contract Tests

**Files:**
- Modify: `post/tests/phase1_contract.rs`

- [ ] Add tests that assert:
  - `ForumStore::search(SearchQuery { q: "sqlx" })` returns post results and highlights `sqlx`.
  - category and tag filters narrow results.
  - hot sorting orders by `score` descending.
  - `post::app::primary_routes()` contains `/search`.
  - `post::app::api_route_inventory()` contains `/api/search`.

### Task 2: Search Domain

**Files:**
- Create: `post/src/domain/search.rs`
- Modify: `post/src/domain/mod.rs`

- [ ] Add `SearchQuery`, `SearchSort`, `SearchResultKind`, `SearchResultItem`, `SearchResultPage`.
- [ ] Add `search_dense_workbench(SearchQuery) -> SearchResultPage`.
- [ ] Reuse homepage seed data through public `dense_workbench_topics()` to avoid duplicating topic records.

### Task 3: Store and API

**Files:**
- Modify: `post/src/state.rs`
- Modify: `post/src/api.rs`
- Modify: `post/src/app.rs`

- [ ] Add `ForumStore::search(query: SearchQuery) -> Result<SearchResultPage, ForumError>`.
- [ ] Add `GET /api/search` with resilient query parsing.
- [ ] Add `/api/search` to API inventory and `/search` to primary routes.

### Task 4: Search Page and Navigation

**Files:**
- Create: `post/src/pages/search.rs`
- Modify: `post/src/pages/mod.rs`
- Modify: `post/src/app.rs`
- Modify: `post/src/components/mod.rs`
- Modify: `post/style/main.css`

- [ ] Add `/search` route.
- [ ] Add `SearchPage` rendering query input, result count, filter controls, and result list.
- [ ] Change top-nav search input into a GET form that submits to `/search`.
- [ ] Keep visual style aligned with Dense Workbench: restrained panels, compact result rows, blue highlight chips.

### Task 5: Verification

**Files:**
- No new files.

- [ ] Run `cargo fmt`.
- [ ] Run `cargo test`.
- [ ] Run `cargo check`.
- [ ] Run `cargo leptos build`.
- [ ] Check IDEA error-level problems for changed Rust/test files.
- [ ] Browser verify `/search?q=sqlx` shows highlighted search results and no horizontal overflow.

## Self-Review

- Covers PRD 4.9 keyword search, category/tag filters, hot/time sort, result highlight, and search result page.
- Keeps Elasticsearch as an explicit future backend replacement while making the current system runnable.
- No placeholders remain.
