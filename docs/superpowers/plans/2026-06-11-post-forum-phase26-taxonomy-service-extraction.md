# Post Forum Phase26 Taxonomy Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move category and tag normalization, validation, update, and merge rules out of `post/src/state.rs` into a focused taxonomy service.

**Architecture:** `ForumStore` continues to own in-memory maps, admin checks, uniqueness checks, and ID generation. `TaxonomyService` owns pure transformations for category/tag construction, update application, and tag merge state changes.

**Tech Stack:** Rust, Leptos SSR project structure, in-memory `ForumStore`, TDD with `cargo test --manifest-path post/Cargo.toml`.

---

### Task 1: Taxonomy Service Rules

**Files:**
- Create: `post/src/services/taxonomy.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Test: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing test**

Add a service-level test that calls `post::services::taxonomy::TaxonomyService` to build and update categories, build and update tags, reject self-merge, add source tag count into target, and disable the merged source tag.

- [x] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path post/Cargo.toml taxonomy_service_normalizes_and_merges_categories_and_tags`

Expected: FAIL because `post::services::taxonomy` is not implemented yet.

- [x] **Step 3: Write minimal implementation**

Create `TaxonomyService` with category/tag builders, update applicators, merge validation, target merge application, and source disable behavior.

- [x] **Step 4: Wire store methods**

Update `ForumStore::create_category`, `update_category`, `create_tag`, `update_tag`, and `merge_tag` to use `TaxonomyService` while leaving admin and uniqueness checks in `state.rs`.

- [x] **Step 5: Verify targeted and full checks**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml taxonomy_service_normalizes_and_merges_categories_and_tags
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands pass.
