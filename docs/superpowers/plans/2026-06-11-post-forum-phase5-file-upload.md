# Post Forum Phase 5 File Upload Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a runnable file upload foundation for PRD 4.2, 6, and 7: validate image MIME/size, store file metadata, produce Markdown-usable URLs, and expose upload metadata through API and editor UI.

**Architecture:** Add `domain::files` for upload request/metadata/limits. `ForumStore` stores file metadata in memory and validates uploader, MIME type, and size. `POST /api/files` accepts JSON metadata for the demo path and returns a `FileAsset` with bucket/key/hash/url fields that can later be backed by RustFS.

**Tech Stack:** Rust 2024, Axum 0.8 JSON API, Leptos 0.8, existing in-memory `ForumStore`.

---

## Scope

This slice does not stream binary multipart content to RustFS yet. It implements the validation and metadata contract needed by the editor, database schema, and later RustFS adapter.

## Tasks

### Task 1: Contract Tests

- [ ] Valid PNG/JPEG/WebP upload returns `FileAsset` with `/uploads/...` URL and Markdown image snippet.
- [ ] Unsupported MIME type is rejected.
- [ ] Oversized file is rejected.
- [ ] API inventory includes `/api/files`.
- [ ] Editor inventory mentions MIME, size, URL, and Markdown insertion.

### Task 2: Domain and Store

- [ ] Create `post/src/domain/files.rs`.
- [ ] Expose module in `domain/mod.rs`.
- [ ] Add `ForumData.files`.
- [ ] Add `ForumStore::upload_file`.

### Task 3: API and Editor

- [ ] Add `POST /api/files`.
- [ ] Add `/api/files` to inventory.
- [ ] Extend editor preview with upload contract and example Markdown image URL.

### Task 4: Verification

- [ ] `cargo fmt`
- [ ] `cargo test`
- [ ] `cargo check`
- [ ] `cargo leptos build`
- [ ] IDEA error check
- [ ] API/browser verify upload contract and editor page

## Self-Review

- Covers PRD file metadata, MIME/size limits, generated URL, Markdown image use, and RustFS-ready shape.
- Leaves binary streaming and actual RustFS persistence as a follow-up adapter.
