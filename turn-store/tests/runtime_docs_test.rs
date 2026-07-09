#[test]
fn runtime_docs_cover_required_services_and_environment() {
    let env = include_str!("../.env.example");
    for entry in [
        "BIND_ADDR=127.0.0.1:3000",
        "DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55432/turn_store",
        "DATABASE_MAX_CONNECTIONS=10",
        "REDIS_URL=redis://127.0.0.1:6379/",
        "REDIS_STREAM_TTL_SECONDS=3600",
        "REDIS_XREAD_BLOCK_MS=15000",
        "RELAY_IDLE_TIMEOUT_SECONDS=60",
        "UPSTREAM_AGENT_URL=http://127.0.0.1:8787/events",
    ] {
        assert!(
            env.lines().any(|line| line == entry),
            ".env.example 缺少精确配置 {entry}"
        );
    }

    let readme = include_str!("../README.md");
    assert!(
        readme.contains(
            r#"docker run --rm -d --name turn-store-postgres \
  -e POSTGRES_PASSWORD=postgres \
  -e POSTGRES_DB=turn_store \
  -p 55432:5432 postgres:17-alpine"#
        ),
        "README 缺少完整 PostgreSQL docker run 命令"
    );
    assert!(
        readme.contains(
            r#"docker run --rm -d --name turn-store-redis \
  -p 6379:6379 redis:7-alpine"#
        ),
        "README 缺少完整 Redis docker run 命令"
    );
    assert!(
        readme.contains("cd turn-store/mock\nnpm install\nnpm run dev"),
        "README 缺少 mock npm run dev 启动顺序"
    );
    assert!(
        readme.contains(
            "cd turn-store\ncp .env.example .env.local\nset -a\nsource .env.local\nset +a\ncargo run"
        ),
        "README 缺少 Axum cargo run 启动顺序"
    );
    assert!(
        readme.contains("http://127.0.0.1:3000/"),
        "README 缺少网页 URL"
    );
    assert!(
        readme.contains("cd turn-store && node --test static/sse.test.mjs"),
        "README 缺少 Node parser 测试命令"
    );
    assert!(
        readme.contains(
            r#"cd turn-store
DATABASE_URL=postgres://postgres:postgres@127.0.0.1:55432/postgres \
TEST_REDIS_URL=redis://127.0.0.1:6380/15 \
cargo test"#
        ),
        "README 缺少完整 cargo test 命令"
    );
}
