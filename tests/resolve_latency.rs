use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use dx::config::AppConfig;
use dx::resolve::{ResolveQuery, Resolver};

fn make_temp_dir(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("dx-it-{label}-{nonce}-{}", std::process::id()));
    fs::create_dir_all(&path).expect("create temp dir");
    path
}

#[test]
fn typical_queries_complete_under_50ms_per_query() {
    let cwd = make_temp_dir("latency");
    fs::create_dir_all(cwd.join("src/components/button")).expect("create tree");

    let resolver = Resolver::with_bookmark_lookup(
        AppConfig {
            search_roots: vec![cwd.clone()],
            ..AppConfig::default()
        },
        |_| None,
    );

    let queries = [".", "src", "src/com/but", "...", "missing"];
    let iterations = 500_u32;

    let started = Instant::now();
    for raw in &queries {
        for _ in 0..iterations {
            let query = ResolveQuery { raw, cwd: &cwd };
            let _ = resolver.resolve(query);
        }
    }
    let elapsed = started.elapsed();
    let total_queries = iterations * queries.len() as u32;
    let per_query = elapsed / total_queries;

    assert!(
        per_query <= Duration::from_millis(50),
        "per-query latency {:?} exceeded 50ms (total {:?})",
        per_query,
        elapsed
    );

    let _ = fs::remove_dir_all(cwd);
}
