use super::{
    abbreviation, is_filesystem_prefix, prepare_candidates, roots,
    strip_filesystem_prefix_for_fallback, traversal, FallbackPolicy, ResolveError, ResolveQuery,
    ResolveResult, Resolver,
};

impl Resolver {
    pub fn resolve(&self, query: ResolveQuery<'_>) -> Result<ResolveResult, ResolveError> {
        let trimmed = query.raw.trim();
        if trimmed.is_empty() {
            return Err(ResolveError::EmptyQuery);
        }

        let mut resolution_query = trimmed;
        let mut uses_prefix_fallback = false;

        if let Some(path) = super::precedence::resolve_direct(query.cwd, trimmed) {
            if path.is_dir() {
                return Ok(ResolveResult { path });
            }

            if is_filesystem_prefix(trimmed) {
                let stripped = strip_filesystem_prefix_for_fallback(trimmed);
                if stripped.is_empty() {
                    return Err(ResolveError::PathNotFound(path.display().to_string()));
                }
                resolution_query = stripped;
                uses_prefix_fallback = true;
            } else {
                return Err(ResolveError::PathNotFound(path.display().to_string()));
            }
        }

        let fallback_policy = FallbackPolicy::from_query_context(
            query.cwd,
            &self.config.search_roots,
            trimmed,
            uses_prefix_fallback,
        );

        if fallback_policy.allow_step_up
            && let Some(path) = traversal::resolve_step_up(query.cwd, resolution_query)
        {
            return Ok(ResolveResult { path });
        }

        let mut candidates = abbreviation::resolve_abbreviation(
            &fallback_policy.effective_roots,
            resolution_query,
            self.config.resolve.case_sensitive,
        );

        if candidates.is_empty() {
            candidates = roots::resolve_fallbacks(
                &fallback_policy.effective_roots,
                resolution_query,
                self.config.resolve.case_sensitive,
            );
        }

        if candidates.is_empty() {
            if fallback_policy.allow_bookmark_lookup
                && let Some(path) = (self.bookmark_lookup)(resolution_query)
            {
                return Ok(ResolveResult { path });
            }
            return Err(ResolveError::NotFound);
        }

        if candidates.len() == 1 {
            return Ok(ResolveResult {
                path: candidates.remove(0),
            });
        }

        prepare_candidates(&mut candidates, None);

        Err(ResolveError::Ambiguous {
            count: candidates.len(),
            candidates,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{bookmarks, config::AppConfig, test_support};

    use super::*;

    fn env_lock() -> std::sync::MutexGuard<'static, ()> {
        test_support::env_lock()
    }

    fn make_temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("dx-{label}-{nonce}-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        path
    }

    fn create_resolver_with_roots(roots: Vec<PathBuf>) -> Resolver {
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
                ..AppConfig::default()
            },
            |_| None,
        )
    }

    fn create_resolver_with_roots_and_bookmarks(roots: Vec<PathBuf>) -> Resolver {
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
                ..AppConfig::default()
            },
            bookmarks::lookup,
        )
    }

    fn set_bookmark_env(file: Option<String>) {
        if let Some(path) = file {
            unsafe { env::set_var("DX_BOOKMARKS_FILE", path) };
        } else {
            unsafe { env::remove_var("DX_BOOKMARKS_FILE") };
        }
    }

    #[test]
    fn resolves_absolute_existing_path() {
        let temp = make_temp_dir("resolve-abs");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: temp.to_str().expect("utf8 path"),
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("resolve");
        assert_eq!(result.path, temp);
    }

    #[test]
    fn resolves_relative_existing_path() {
        let temp = make_temp_dir("resolve-rel");
        let child = temp.join("src");
        fs::create_dir_all(&child).expect("create dir");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "./src",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("resolve");
        assert_eq!(result.path, child);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn errors_on_nonexistent_path() {
        let temp = make_temp_dir("resolve-miss");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        let query = ResolveQuery {
            raw: "./does-not-exist",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query)
            .expect_err("should error");
        assert!(matches!(err, ResolveError::NotFound));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_leading_slash_direct_miss_falls_back_from_filesystem_root() {
        let temp = make_temp_dir("resolve-leading-slash-root");
        let canonical_temp = fs::canonicalize(&temp).expect("canonical temp dir");
        let missing_prefix = format!("dx-root-only-{}", std::process::id());
        let target = canonical_temp.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query_string = format!("{}/{}{}", canonical_temp.display(), missing_prefix, "/pro");
        let query = ResolveQuery {
            raw: &query_string,
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_leading_slash_direct_miss_does_not_use_bookmark_lookup() {
        let temp = make_temp_dir("resolve-leading-slash-no-bookmark");
        let missing_prefix = format!("dx-bookmark-only-{}", std::process::id());
        let resolver = Resolver::with_bookmark_lookup(AppConfig::default(), |_| Some(PathBuf::from("/tmp")));

        let query_string = format!("/{missing_prefix}/pro");
        let query = ResolveQuery {
            raw: &query_string,
            cwd: &temp,
        };

        let err = resolver
            .resolve(query)
            .expect_err("leading slash fallback should skip bookmarks");
        assert!(matches!(err, ResolveError::NotFound));

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_dot_slash_direct_miss_falls_back_to_abbreviation() {
        let temp = make_temp_dir("resolve-dot-slash-fallback");
        let root = temp.join("root");
        let target = root.join("no-local-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "./no-local-hit/pro",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_tilde_slash_direct_miss_falls_back_to_abbreviation() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-tilde-slash-fallback");
        let home = temp.join("home");
        fs::create_dir_all(&home).expect("create home");

        let root = temp.join("root");
        let target = root.join("no-home-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let prev_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &home) };

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "~/no-home-hit/pro",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("fallback should resolve");
        assert_eq!(result.path, target);

        if let Some(value) = prev_home {
            unsafe { std::env::set_var("HOME", value) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn resolve_prefixed_empty_fallback_query_preserves_path_not_found() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-empty-prefixed-fallback");
        let missing_home = temp.join("missing-home");

        let prev_home = std::env::var("HOME").ok();
        unsafe { std::env::set_var("HOME", &missing_home) };

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "~/",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query)
            .expect_err("missing home directory should keep path-not-found");
        assert!(matches!(err, ResolveError::PathNotFound(_)));

        if let Some(value) = prev_home {
            unsafe { std::env::set_var("HOME", value) };
        } else {
            unsafe { std::env::remove_var("HOME") };
        }

        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn returns_ambiguous_error_for_multiple_candidates() {
        let temp = make_temp_dir("resolve-ambiguous");
        let root = temp.join("root");
        fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
        fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "pro/al",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query)
            .expect_err("should be ambiguous");
        assert!(matches!(
            err,
            ResolveError::Ambiguous {
                count: 2,
                candidates: _
            }
        ));
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn direct_resolution_wins_over_fallback_search_root() {
        let temp = make_temp_dir("resolve-precedence");
        let local = temp.join("src");
        fs::create_dir_all(&local).expect("create local src");

        let root = temp.join("root");
        fs::create_dir_all(root.join("src")).expect("create fallback src");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "src",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("should resolve local");
        assert_eq!(result.path, local);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn bookmark_resolves_when_no_filesystem_match_exists() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");
        let bookmark_target = temp.join("target");
        fs::create_dir_all(&bookmark_target).expect("create bookmark target");

        let canonical_target = fs::canonicalize(&bookmark_target).expect("canonical target");
        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            canonical_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("bookmark should resolve");
        assert_eq!(result.path, canonical_target);

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn fallback_root_takes_precedence_over_bookmark() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-fallback-over-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");

        let fallback_root = temp.join("root");
        let fallback_match = fallback_root.join("proj");
        fs::create_dir_all(&fallback_match).expect("create fallback match");

        let bookmark_target = temp.join("bookmark-target");
        fs::create_dir_all(&bookmark_target).expect("create bookmark target");
        let canonical_bookmark = fs::canonicalize(&bookmark_target).expect("canonical bookmark");
        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            canonical_bookmark
                .display()
                .to_string()
                .replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![fallback_root]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let result = resolver
            .resolve(query)
            .expect("fallback should resolve");
        assert_eq!(result.path, fallback_match);

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn stale_bookmark_returns_no_match_and_resolution_fails() {
        let _guard = env_lock();
        let temp = make_temp_dir("resolve-stale-bookmark");
        let bookmarks_file = temp.join("bookmarks.toml");
        let missing_target = temp.join("missing-target");

        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            missing_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        set_bookmark_env(Some(bookmarks_file.display().to_string()));

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: &temp,
        };

        let err = resolver
            .resolve(query)
            .expect_err("stale bookmark should fail");
        assert!(matches!(err, ResolveError::NotFound));

        set_bookmark_env(None);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn effective_roots_include_cwd_when_no_roots_configured() {
        let temp = make_temp_dir("effective-roots-cwd");
        let roots = super::super::build_effective_roots(&temp, &[]);
        assert_eq!(roots, vec![temp.clone()]);
        let _ = fs::remove_dir_all(temp);
    }

    #[test]
    fn effective_roots_dedup_when_cwd_already_configured() {
        let temp = make_temp_dir("effective-roots-dedup");
        let roots = super::super::build_effective_roots(&temp, std::slice::from_ref(&temp));
        assert_eq!(roots, vec![temp.clone()]);
        let _ = fs::remove_dir_all(temp);
    }
}
