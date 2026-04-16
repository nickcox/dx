use std::time::{Duration, Instant};

use dx::config::AppConfig;
use dx::resolve::{ResolveQuery, Resolver};

fn main() {
    let cwd = std::env::current_dir().expect("cwd");
    let resolver = Resolver::with_bookmark_lookup(
        AppConfig {
            search_roots: vec![cwd.clone()],
            ..AppConfig::default()
        },
        |_| None,
    );

    let sample_queries = [".", "..", "~"];

    let iterations = 2_000_u32;
    let started = Instant::now();
    for _ in 0..iterations {
        for raw in &sample_queries {
            let query = ResolveQuery { raw, cwd: &cwd };
            let _ = resolver.resolve(query);
        }
    }
    let elapsed = started.elapsed();
    let per_query = elapsed / (iterations * sample_queries.len() as u32);

    println!(
        "resolve latency benchmark: total={:?}, per_query={:?}",
        elapsed, per_query
    );

    let threshold = Duration::from_millis(50);
    if per_query > threshold {
        eprintln!(
            "per-query latency {:?} exceeded threshold {:?}",
            per_query, threshold
        );
        std::process::exit(1);
    }
}
