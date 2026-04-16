use serde::Serialize;

use super::{ResolveError, ResolveMode, ResolveQuery, Resolver};

#[derive(Debug, Serialize)]
struct JsonOutput<'a> {
    status: &'a str,
    reason: Option<&'a str>,
    path: Option<String>,
    candidates: Option<Vec<String>>,
}

impl Resolver {
    pub fn execute(&self, raw_query: &str, mode: ResolveMode) -> i32 {
        let cwd = match std::env::current_dir() {
            Ok(path) => path,
            Err(err) => {
                eprintln!("dx resolve: failed to read current directory: {err}");
                return 1;
            }
        };

        let query = ResolveQuery {
            raw: raw_query,
            cwd: &cwd,
        };

        match self.resolve(query) {
            Ok(result) => match mode {
                ResolveMode::Default => {
                    println!("{}", result.path.display());
                    0
                }
                ResolveMode::List => {
                    println!("{}", result.path.display());
                    0
                }
                ResolveMode::Json => {
                    let payload = JsonOutput {
                        status: "ok",
                        reason: None,
                        path: Some(result.path.display().to_string()),
                        candidates: None,
                    };
                    match serde_json::to_string(&payload) {
                        Ok(json) => {
                            println!("{json}");
                            0
                        }
                        Err(err) => {
                            eprintln!("dx resolve: failed to serialize json: {err}");
                            1
                        }
                    }
                }
            },
            Err(err) => self.emit_error(err, mode),
        }
    }

    fn emit_error(&self, err: ResolveError, mode: ResolveMode) -> i32 {
        match (mode, err) {
            (ResolveMode::Json, ResolveError::Ambiguous { candidates, .. }) => {
                let payload = JsonOutput {
                    status: "error",
                    reason: Some("ambiguous"),
                    path: None,
                    candidates: Some(
                        candidates
                            .into_iter()
                            .map(|path| path.display().to_string())
                            .collect(),
                    ),
                };
                match serde_json::to_string(&payload) {
                    Ok(json) => {
                        println!("{json}");
                        0
                    }
                    Err(serialization_error) => {
                        eprintln!("dx resolve: failed to serialize json: {serialization_error}");
                        1
                    }
                }
            }
            (ResolveMode::Json, ResolveError::NotFound) => {
                let payload = JsonOutput {
                    status: "error",
                    reason: Some("not_found"),
                    path: None,
                    candidates: None,
                };
                match serde_json::to_string(&payload) {
                    Ok(json) => {
                        println!("{json}");
                        1
                    }
                    Err(serialization_error) => {
                        eprintln!("dx resolve: failed to serialize json: {serialization_error}");
                        1
                    }
                }
            }
            (ResolveMode::List, ResolveError::Ambiguous { candidates, .. }) => {
                for candidate in candidates {
                    println!("{}", candidate.display());
                }
                0
            }
            (_, ResolveError::Ambiguous { candidates, .. }) => {
                eprintln!("dx resolve: ambiguous query; candidates:");
                for candidate in candidates {
                    eprintln!("- {}", candidate.display());
                }
                1
            }
            (_, other) => {
                eprintln!("dx resolve: {other}");
                1
            }
        }
    }
}
