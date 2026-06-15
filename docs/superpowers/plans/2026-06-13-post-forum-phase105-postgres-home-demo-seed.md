# Post Forum Phase 105 PostgreSQL Home Demo Seed

**Goal:** Keep the local PostgreSQL-backed homepage demo aligned with the Dense Workbench design after live tests have populated the development database.

## Scope

- Add a local demo seed switch for the homepage design data.
- Persist the 12 design-reference topics into PostgreSQL at SSR startup.
- Keep the seed backend-side and SQLx-backed instead of hardcoding homepage rows in the frontend.
- Preserve the design pagination label for the default local homepage.
- Verify the real browser homepage matches the design-reference content.

## Tasks

- [x] Add a RED contract for local PostgreSQL homepage demo seed wiring.
- [x] Add `PostgresDemoSeedRepository::ensure_homepage_seed`.
- [x] Use `sqlx::query!` for users, categories, tags, posts, contents, and post-tags seed writes.
- [x] Add `POST_DEMO_SEED_HOME=true` to local environment templates.
- [x] Call the seed during SSR startup when enabled.
- [x] Sort recommended posts before ordinary rows so demo rows remain stable in a polluted local dev database.
- [x] Preserve the design pagination total for the default local demo homepage.
- [x] Verify the homepage in the in-app browser.

## Verification

- RED:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract postgres_runtime_can_seed_dense_workbench_home_for_local_demo -- --nocapture`: failed before implementation because `src/repositories/demo_seed.rs` did not exist.
- GREEN:
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract postgres_runtime_can_seed_dense_workbench_home_for_local_demo -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract app_state_home_runtime_supports_demo_fallback -- --nocapture`: PASS.
  - `env CARGO_INCREMENTAL=0 cargo test --manifest-path post/Cargo.toml --test phase1_contract sqlx_post_repository_contract_maps_homepage_post_rows -- --nocapture`: PASS.
- Browser:
  - `http://127.0.0.1:3000/` matched all 12 design-reference topic titles.
  - The page contained all four sidebar headings: 分类、热门标签、公告、活跃作者.
  - The page contained `显示 1-12 / 342 个主题`.
  - The page did not contain the removed `系统功能` card.
  - `documentElement.scrollWidth <= documentElement.clientWidth`.

## PRD Coverage

- Supports PRD `4.1.1` by making the homepage visible content match the provided design.
- Supports PRD `4.1.2` by ensuring PostgreSQL runtime data can still reproduce the design seed instead of relying on frontend-only mock rows.
