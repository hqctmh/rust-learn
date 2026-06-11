# Post Forum Phase 19 Post Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move author post creation/update normalization rules out of `state.rs` into `services::posts` while preserving the existing public API behavior.

**Architecture:** `state.rs` remains responsible for locking, identity checks, ID allocation, persistence into the in-memory maps, and notification side effects. `services::posts` owns pure rules: validation, summary normalization, tag normalization, safe Markdown rendering, building a new `PostDetail`, and applying editor changes to an existing `PostDetail`.

**Tech Stack:** Rust, existing domain post models, existing `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Post Authoring Rules

**Files:**
- Create/modify: `post/src/services/posts.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::posts::PostAuthoringService` and verifies:
- blank title is rejected;
- markdown is HTML-escaped;
- tags are lowercased, `#` is stripped, and duplicates are removed;
- empty summary falls back to the first Markdown line;
- update can publish a draft and set `published_at`.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml post_authoring_service
```

Expected: compile failure because `services::posts` does not exist.

- [x] **Step 3: Implement `PostAuthoringService`**

Create methods for validating input, building new post details, and applying editor updates.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline normalization/rendering/build logic in `autosave_draft`, `update_post`, and `create_post` with `PostAuthoringService`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
