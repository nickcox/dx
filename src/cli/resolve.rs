use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::resolve::{ResolveError, ResolveQuery, Resolver};

use super::CliError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResolveMode {
    Default,
    List,
    Json,
}

#[derive(Debug, Serialize)]
struct JsonOutput<'a> {
    status: &'a str,
    reason: Option<&'a str>,
    path: Option<String>,
    candidates: Option<Vec<String>>,
}

pub fn run_resolve(
    resolver: &Resolver,
    query: &str,
    list: bool,
    json: bool,
) -> Result<(), CliError> {
    let mode = if json {
        ResolveMode::Json
    } else if list {
        ResolveMode::List
    } else {
        ResolveMode::Default
    };

    let cwd = std::env::current_dir().map_err(CliError::ResolveCurrentDir)?;

    match resolver.resolve(ResolveQuery {
        raw: query,
        cwd: &cwd,
    }) {
        Ok(result) => emit_success(&result.path, mode),
        Err(error) => emit_error(error, mode),
    }
}

fn emit_success(path: &Path, mode: ResolveMode) -> Result<(), CliError> {
    match mode {
        ResolveMode::Default | ResolveMode::List => println!("{}", path.display()),
        ResolveMode::Json => println!(
            "{}",
            to_json(&JsonOutput {
                status: "ok",
                reason: None,
                path: Some(path.display().to_string()),
                candidates: None,
            })?
        ),
    }

    Ok(())
}

/// Renders a failed resolution.
///
/// Ambiguity is a reportable outcome rather than a failure in the
/// machine-readable modes, so those arms write to stdout and return `Ok`.
fn emit_error(error: ResolveError, mode: ResolveMode) -> Result<(), CliError> {
    match (mode, error) {
        (ResolveMode::Json, ResolveError::Ambiguous { candidates, .. }) => {
            println!(
                "{}",
                to_json(&JsonOutput {
                    status: "error",
                    reason: Some("ambiguous"),
                    path: None,
                    candidates: Some(display_all(&candidates)),
                })?
            );
            Ok(())
        }
        (ResolveMode::List, ResolveError::Ambiguous { candidates, .. }) => {
            for candidate in candidates {
                println!("{}", candidate.display());
            }
            Ok(())
        }
        (ResolveMode::Json, ResolveError::NotFound) => {
            println!(
                "{}",
                to_json(&JsonOutput {
                    status: "error",
                    reason: Some("not_found"),
                    path: None,
                    candidates: None,
                })?
            );
            Err(CliError::ResolveReportedAsJson)
        }
        (_, ResolveError::Ambiguous { candidates, .. }) => {
            Err(CliError::AmbiguousResolve(candidates))
        }
        (_, other) => Err(CliError::Resolve(other)),
    }
}

fn to_json(payload: &JsonOutput<'_>) -> Result<String, CliError> {
    serde_json::to_string(payload).map_err(CliError::ResolveJson)
}

fn display_all(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .map(|path| path.display().to_string())
        .collect()
}
