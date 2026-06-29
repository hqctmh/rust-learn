#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use std::env;
#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use std::fs;
#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use std::hint::black_box;
#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use std::path::{Path, PathBuf};

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use tikv_jemalloc_ctl::{epoch, profiling, stats};

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: Jemalloc = Jemalloc;

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
#[allow(non_upper_case_globals)]
#[unsafe(export_name = "_rjem_malloc_conf")]
pub static malloc_conf: &[u8] = b"prof:true,prof_active:true,lg_prof_sample:19\0";

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
const DEFAULT_PROFILE_PATH: &str = "heap.pb.gz";

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
const CHUNK_SIZE: usize = 1024 * 1024;

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
const CHUNK_COUNT: usize = 128;

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let profile_path = env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_PROFILE_PATH));

    print_jemalloc_config()?;
    activate_profiling().await?;

    println!(
        "allocating {} MiB demo heap...",
        bytes_to_mib(CHUNK_SIZE * CHUNK_COUNT)
    );
    let data = allocate_demo_heap();
    black_box(&data);

    print_jemalloc_stats("after demo allocation")?;
    write_pprof(&profile_path).await?;

    println!("heap profile dumped to: {}", profile_path.display());
    println!(
        "serve with: scripts/serve_heap_pprof.sh {} 127.0.0.1:8080",
        profile_path.display()
    );

    black_box(data);
    Ok(())
}

#[cfg(not(all(not(target_env = "msvc"), target_os = "linux")))]
fn main() {
    eprintln!("jemalloc_pprof demo only supports Linux targets.");
    eprintln!("On Linux run: cargo run --bin jemalloc_profile_demo -- heap.pb.gz");
    eprintln!("Then serve it with: scripts/serve_heap_pprof.sh heap.pb.gz 127.0.0.1:8080");
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
async fn activate_profiling() -> anyhow::Result<()> {
    let prof_ctl = jemalloc_pprof::PROF_CTL
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("jemalloc profiling is disabled"))?;

    let mut prof_ctl = prof_ctl.lock().await;
    if !prof_ctl.activated() {
        prof_ctl
            .activate()
            .map_err(|err| anyhow::anyhow!("failed to activate jemalloc profiling: {err:?}"))?;
    }

    Ok(())
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
async fn write_pprof(path: &Path) -> anyhow::Result<()> {
    let prof_ctl = jemalloc_pprof::PROF_CTL
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("jemalloc profiling is disabled"))?;

    let mut prof_ctl = prof_ctl.lock().await;
    let profile = prof_ctl
        .dump_pprof()
        .map_err(|err| anyhow::anyhow!("failed to dump jemalloc pprof profile: {err}"))?;

    fs::write(path, profile)
        .map_err(|err| anyhow::anyhow!("failed to write {}: {err}", path.display()))?;

    Ok(())
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn allocate_demo_heap() -> Vec<Vec<u8>> {
    (0..CHUNK_COUNT)
        .map(|index| allocate_chunk((index % u8::MAX as usize) as u8))
        .collect()
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn allocate_chunk(fill: u8) -> Vec<u8> {
    vec![fill; CHUNK_SIZE]
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn print_jemalloc_config() -> anyhow::Result<()> {
    println!(
        "jemalloc opt.prof={}, opt.lg_prof_sample={}",
        read_ctl("opt.prof", profiling::prof::read)?,
        read_ctl("opt.lg_prof_sample", profiling::lg_prof_sample::read)?
    );

    Ok(())
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn print_jemalloc_stats(label: &str) -> anyhow::Result<()> {
    epoch::advance().map_err(|err| anyhow::anyhow!("failed to refresh jemalloc stats: {err:?}"))?;

    println!(
        "{label}: allocated={} MiB, active={} MiB, resident={} MiB, mapped={} MiB",
        bytes_to_mib(
            stats::allocated::read()
                .map_err(|err| anyhow::anyhow!("failed to read stats.allocated: {err:?}"))?
        ),
        bytes_to_mib(
            stats::active::read()
                .map_err(|err| anyhow::anyhow!("failed to read stats.active: {err:?}"))?
        ),
        bytes_to_mib(
            stats::resident::read()
                .map_err(|err| anyhow::anyhow!("failed to read stats.resident: {err:?}"))?
        ),
        bytes_to_mib(
            stats::mapped::read()
                .map_err(|err| anyhow::anyhow!("failed to read stats.mapped: {err:?}"))?
        ),
    );

    Ok(())
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn bytes_to_mib(bytes: usize) -> usize {
    bytes / 1024 / 1024
}

#[cfg(all(not(target_env = "msvc"), target_os = "linux"))]
fn read_ctl<T, F>(name: &str, read: F) -> anyhow::Result<T>
where
    F: FnOnce() -> tikv_jemalloc_ctl::Result<T>,
{
    read().map_err(|err| anyhow::anyhow!("failed to read {name}: {err:?}"))
}
