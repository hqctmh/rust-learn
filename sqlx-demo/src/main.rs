use std::env;
use std::hint::black_box;
use std::sync::{Arc, Mutex};

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::StatusCode;
#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use axum::http::{HeaderValue, header};
#[cfg(not(all(not(target_env = "msvc"), target_os = "linux")))]
use axum::response::IntoResponse;
#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

#[cfg(not(target_env = "msvc"))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: Jemalloc = Jemalloc;

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

const BYTES_PER_MIB: usize = 1024 * 1024;
const DEFAULT_MIB: usize = 4;
const MAX_MIB_PER_REQUEST: usize = 64;
const DEFAULT_REPEAT: usize = 1;
const MAX_REPEAT: usize = 32;

type SharedState = Arc<AppState>;

#[derive(Default)]
struct AppState {
    byte_leaks: Mutex<Vec<Vec<u8>>>,
    string_leaks: Mutex<Vec<String>>,
}

#[derive(Deserialize)]
struct LoadQuery {
    mib: Option<usize>,
    repeat: Option<usize>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DemoResponse {
    endpoint: &'static str,
    allocated_mib: usize,
    retained_mib: usize,
    byte_leak_chunks: usize,
    string_leak_chunks: usize,
    note: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    byte_leak_chunks: usize,
    string_leak_chunks: usize,
    retained_mib: usize,
    routes: [&'static str; 5],
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let state = Arc::new(AppState::default());

    let app = Router::new()
        .route("/v1/demo/health", get(health))
        .route("/v1/demo/cpu", post(cpu_work))
        .route("/v1/demo/transient-bytes", post(transient_bytes))
        .route("/v1/demo/leak/bytes", post(leak_bytes))
        .route("/v1/demo/leak/strings", post(leak_strings))
        .merge(debug_router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    println!("axum memory demo listening on http://{bind_addr}");
    println!("heap profile endpoint: http://{bind_addr}/debug/pprof/heap");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<SharedState>) -> Json<HealthResponse> {
    let byte_leak_chunks = lock_len(&state.byte_leaks);
    let string_leak_chunks = lock_len(&state.string_leaks);

    Json(HealthResponse {
        status: "ok",
        byte_leak_chunks,
        string_leak_chunks,
        retained_mib: retained_mib(&state),
        routes: [
            "GET /v1/demo/health",
            "POST /v1/demo/cpu",
            "POST /v1/demo/transient-bytes?mib=4&repeat=1",
            "POST /v1/demo/leak/bytes?mib=4&repeat=1",
            "POST /v1/demo/leak/strings?mib=4&repeat=1",
        ],
    })
}

async fn cpu_work(
    State(state): State<SharedState>,
    Query(query): Query<LoadQuery>,
) -> Json<DemoResponse> {
    let repeat = repeat(&query);
    let mut checksum = 0_u64;

    for n in 0..repeat {
        checksum = checksum.wrapping_add(fibonacci(28 + (n % 4) as u64));
    }
    black_box(checksum);

    Json(response(
        "cpu",
        0,
        &state,
        "CPU work only; it should not retain heap memory.",
    ))
}

async fn transient_bytes(
    State(state): State<SharedState>,
    Query(query): Query<LoadQuery>,
) -> Json<DemoResponse> {
    let mib = mib(&query);
    let repeat = repeat(&query);
    let allocated_mib = mib * repeat;

    for index in 0..repeat {
        let chunk = allocate_bytes(mib, index as u8);
        black_box(&chunk);
    }

    Json(response(
        "transient_bytes",
        allocated_mib,
        &state,
        "Allocated bytes were dropped before the response.",
    ))
}

async fn leak_bytes(
    State(state): State<SharedState>,
    Query(query): Query<LoadQuery>,
) -> Json<DemoResponse> {
    let mib = mib(&query);
    let repeat = repeat(&query);
    let allocated_mib = mib * repeat;

    let mut byte_leaks = state.byte_leaks.lock().expect("byte leak mutex poisoned");
    for index in 0..repeat {
        byte_leaks.push(allocate_bytes(mib, index as u8));
    }
    drop(byte_leaks);

    Json(response(
        "leak_bytes",
        allocated_mib,
        &state,
        "Allocated byte chunks were retained in AppState.",
    ))
}

async fn leak_strings(
    State(state): State<SharedState>,
    Query(query): Query<LoadQuery>,
) -> Json<DemoResponse> {
    let mib = mib(&query);
    let repeat = repeat(&query);
    let allocated_mib = mib * repeat;

    let mut string_leaks = state
        .string_leaks
        .lock()
        .expect("string leak mutex poisoned");
    for index in 0..repeat {
        string_leaks.push(allocate_string(mib, index));
    }
    drop(string_leaks);

    Json(response(
        "leak_strings",
        allocated_mib,
        &state,
        "Allocated strings were retained in AppState.",
    ))
}

fn response(
    endpoint: &'static str,
    allocated_mib: usize,
    state: &AppState,
    note: &'static str,
) -> DemoResponse {
    DemoResponse {
        endpoint,
        allocated_mib,
        retained_mib: retained_mib(state),
        byte_leak_chunks: lock_len(&state.byte_leaks),
        string_leak_chunks: lock_len(&state.string_leaks),
        note,
    }
}

fn mib(query: &LoadQuery) -> usize {
    query
        .mib
        .unwrap_or(DEFAULT_MIB)
        .clamp(1, MAX_MIB_PER_REQUEST)
}

fn repeat(query: &LoadQuery) -> usize {
    query.repeat.unwrap_or(DEFAULT_REPEAT).clamp(1, MAX_REPEAT)
}

fn allocate_bytes(mib: usize, fill: u8) -> Vec<u8> {
    vec![fill; mib * BYTES_PER_MIB]
}

fn allocate_string(mib: usize, index: usize) -> String {
    let mut value = String::with_capacity(mib * BYTES_PER_MIB);
    let line = format!("leaked-string-{index:04}-");

    while value.len() + line.len() <= value.capacity() {
        value.push_str(&line);
    }

    value
}

fn fibonacci(n: u64) -> u64 {
    match n {
        0 | 1 => n,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

fn retained_mib(state: &AppState) -> usize {
    let byte_mib = state
        .byte_leaks
        .lock()
        .expect("byte leak mutex poisoned")
        .iter()
        .map(|chunk| chunk.capacity())
        .sum::<usize>()
        / BYTES_PER_MIB;

    let string_mib = state
        .string_leaks
        .lock()
        .expect("string leak mutex poisoned")
        .iter()
        .map(|chunk| chunk.capacity())
        .sum::<usize>()
        / BYTES_PER_MIB;

    byte_mib + string_mib
}

fn lock_len<T>(values: &Mutex<Vec<T>>) -> usize {
    values.lock().expect("leak mutex poisoned").len()
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn debug_router() -> Router<SharedState> {
    Router::new().route("/debug/pprof/heap", get(heap_profile))
}

#[cfg(not(all(not(target_env = "msvc"), target_os = "linux")))]
fn debug_router() -> Router<SharedState> {
    Router::new().route("/debug/pprof/heap", get(heap_profile_unavailable))
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
async fn heap_profile() -> Result<Response, (StatusCode, String)> {
    let prof_ctl = jemalloc_pprof::PROF_CTL.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "jemalloc profiling is disabled".to_string(),
        )
    })?;

    let mut prof_ctl = prof_ctl.lock().await;
    if !prof_ctl.activated() {
        prof_ctl
            .activate()
            .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, format!("{err:?}")))?;
    }

    let profile = prof_ctl
        .dump_pprof()
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

    Response::builder()
        .header(header::CONTENT_TYPE, "application/octet-stream")
        .header(
            header::CONTENT_DISPOSITION,
            HeaderValue::from_static("attachment; filename=\"heap.pb.gz\""),
        )
        .body(Body::from(profile))
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

#[cfg(not(all(not(target_env = "msvc"), target_os = "linux")))]
async fn heap_profile_unavailable() -> impl IntoResponse {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        "jemalloc_pprof heap endpoint only supports Linux targets",
    )
}
