use std::env;

use anyhow::{Context, Result, anyhow};

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    pub redis_stream_ttl_seconds: u64,
    pub redis_xread_block_ms: usize,
    pub upstream_agent_url: String,
    pub relay_idle_timeout_seconds: u64,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            bind_addr: env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string()),
            database_url: env::var("DATABASE_URL").context(
                "缺少环境变量 DATABASE_URL，例如 postgres://postgres:postgres@127.0.0.1/turn_store",
            )?,
            database_max_connections: parse_env("DATABASE_MAX_CONNECTIONS", 10)?,
            redis_url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1/".to_string()),
            redis_stream_ttl_seconds: parse_env("REDIS_STREAM_TTL_SECONDS", 3600)?,
            redis_xread_block_ms: parse_env("REDIS_XREAD_BLOCK_MS", 15_000)?,
            upstream_agent_url: env::var("UPSTREAM_AGENT_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8787/events".to_string()),
            relay_idle_timeout_seconds: parse_env("RELAY_IDLE_TIMEOUT_SECONDS", 60)?,
        })
    }
}

fn parse_env<T>(name: &str, default: T) -> Result<T>
where
    T: std::str::FromStr + ToString,
    T::Err: std::fmt::Display,
{
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    value
        .parse()
        .map_err(|error| anyhow!("环境变量 {name}={value} 无法解析: {error}"))
}
