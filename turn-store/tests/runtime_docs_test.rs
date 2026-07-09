#[test]
fn runtime_docs_cover_required_services_and_environment() {
    let env = include_str!("../.env.example");
    for name in [
        "DATABASE_URL",
        "REDIS_URL",
        "UPSTREAM_AGENT_URL",
        "RELAY_IDLE_TIMEOUT_SECONDS",
    ] {
        assert!(env.contains(name), "缺少环境变量 {name}");
    }

    let readme = include_str!("../README.md");
    for command in [
        "docker run",
        "npm run dev",
        "cargo run",
        "node --test static/sse.test.mjs",
    ] {
        assert!(readme.contains(command), "README 缺少命令 {command}");
    }
}
