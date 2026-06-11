# Post Forum Phase 23 Report Service Extraction Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move report creation and handling rules out of `state.rs` into `services::reports`.

**Architecture:** `state.rs` remains responsible for locking, reporter/admin lookup, report id allocation, and target-title resolution from in-memory data. `services::reports` owns pure rules for validating report requests, trimming fields, building `ReportItem`, and applying admin handling state.

**Tech Stack:** Rust, existing report domain models, existing in-memory `ForumStore`, no new third-party dependencies.

---

### Task 1: Extract Report Rules

**Files:**
- Create/modify: `post/src/services/reports.rs`
- Modify: `post/src/services/mod.rs`
- Modify: `post/src/state.rs`
- Modify: `post/tests/phase1_contract.rs`

- [x] **Step 1: Write the failing service boundary test**

Add a test that imports `post::services::reports::ReportService` and verifies:
- blank reason is rejected;
- report creation trims reason and drops blank description;
- created report starts in `Pending`;
- handling a report sets status, handler fields, note, and handled timestamp;
- handling cannot set status back to `Pending`.

- [x] **Step 2: Run the service test to verify RED**

Run:

```bash
cargo test --manifest-path post/Cargo.toml report_service
```

Expected: compile failure because `services::reports` does not exist.

- [x] **Step 3: Implement `ReportService`**

Create methods for building reports and applying admin handling.

- [x] **Step 4: Wire `state.rs` to the service**

Replace inline report construction and handling updates in `ForumStore::create_report` and `ForumStore::handle_report`.

- [x] **Step 5: Run verification**

Run:

```bash
cargo fmt --manifest-path post/Cargo.toml
cargo test --manifest-path post/Cargo.toml
cargo check --manifest-path post/Cargo.toml
cd post && cargo leptos build
```

Expected: all commands exit 0.
