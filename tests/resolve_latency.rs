mod common;

use std::fs;
use std::time::{Duration, Instant};

use dx::config::AppConfig;
use dx::resolve::{ResolveQuery, Resolver};

#[test]
fn typical_queries_complete_under_50ms_per_query() {
    let cwd = common::temp_dir("latency");
    fs::create_dir_all(cwd.path().join("src/components/button")).expect("create tree");

    let resolver = Resolver::with_bookmark_lookup(
        AppConfig {
            search_roots: vec![cwd.path().to_path_buf()],
            ..AppConfig::default()
        },
        |_| None,
    );

    let queries = [".", "src", "src/com/but", "...", "missing"];
    let iterations = 500_u32;

    let started = Instant::now();
    for raw in &queries {
        for _ in 0..iterations {
            let query = ResolveQuery {
                raw,
                cwd: cwd.path(),
            };
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
}
