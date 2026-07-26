use super::{
    FilesystemPrefixFallback, ResolveError, ResolveQuery, ResolveResult, Resolver,
    prepare_candidates, prepare_search_query, resolve_search_candidates, traversal,
    traversal::OnIoError,
};

impl Resolver {
    pub fn resolve(&self, query: ResolveQuery<'_>) -> Result<ResolveResult, ResolveError> {
        let prepared = prepare_search_query(
            query.cwd,
            &self.config.search_roots,
            query.raw,
            FilesystemPrefixFallback::DirectResolutionOnly,
        )?;

        if let Some(path) = prepared.direct_dir {
            return Ok(ResolveResult { path });
        }

        if prepared.fallback_policy.allow_step_up
            && let Some(path) = traversal::resolve_step_up(query.cwd, &prepared.effective_query)
        {
            return Ok(ResolveResult { path });
        }

        // Resolution must not silently drop candidates it could not read: a
        // narrowed set could turn a genuine ambiguity into a confident answer.
        let mut candidates = resolve_search_candidates(
            &prepared.fallback_policy.effective_roots,
            &prepared.effective_query,
            self.config.resolve.case_sensitive,
            OnIoError::Propagate,
        )?;

        if candidates.is_empty() {
            if prepared.fallback_policy.allow_bookmark_lookup
                && let Some(path) = (self.bookmark_lookup)(&prepared.effective_query)
            {
                return Ok(ResolveResult { path });
            }
            return Err(ResolveError::NotFound);
        }

        // Deduplicate before counting, so overlapping roots can never report a
        // single destination as ambiguous.
        prepare_candidates(&mut candidates, None);

        if candidates.len() == 1 {
            return Ok(ResolveResult {
                path: candidates.remove(0),
            });
        }

        Err(ResolveError::Ambiguous { candidates })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::{bookmarks, config::AppConfig, test_support};

    use super::*;

    fn create_resolver_with_roots(roots: Vec<PathBuf>) -> Resolver {
        create_resolver_with_roots_and_case_sensitivity(roots, true)
    }

    fn create_resolver_with_roots_and_case_sensitivity(
        roots: Vec<PathBuf>,
        case_sensitive: bool,
    ) -> Resolver {
        Resolver::with_bookmark_lookup(
            AppConfig {
                search_roots: roots,
                resolve: crate::config::ResolveOptions { case_sensitive },
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

    #[test]
    fn resolves_absolute_existing_path() {
        let temp = test_support::temp_dir("resolve-abs");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: temp.path().to_str().expect("utf8 path"),
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("resolve");
        assert_eq!(result.path, temp.path());
    }

    #[test]
    fn resolves_relative_existing_path() {
        let temp = test_support::temp_dir("resolve-rel");
        let child = temp.path().join("src");
        fs::create_dir_all(&child).expect("create dir");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "./src",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("resolve");
        assert_eq!(result.path, child);
    }

    #[test]
    fn resolves_direct_path_with_significant_whitespace() {
        let temp = test_support::temp_dir("resolve-whitespace");
        let child = temp.path().join("project notes");
        fs::create_dir_all(&child).expect("create dir");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let result = resolver
            .resolve(ResolveQuery {
                raw: "./project notes",
                cwd: temp.path(),
            })
            .expect("resolve");

        assert_eq!(
            fs::canonicalize(result.path).expect("canonical result"),
            fs::canonicalize(child).expect("canonical expected")
        );
    }

    #[cfg(unix)]
    #[test]
    fn resolves_unix_backslash_filename_without_splitting_it() {
        let temp = test_support::temp_dir("resolve-backslash-name");
        let child = temp.path().join(r"project\source");
        fs::create_dir_all(&child).expect("create dir");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let result = resolver
            .resolve(ResolveQuery {
                raw: r"project\source",
                cwd: temp.path(),
            })
            .expect("resolve");

        assert_eq!(
            fs::canonicalize(result.path).expect("canonical result"),
            fs::canonicalize(child).expect("canonical expected")
        );
    }

    #[cfg(windows)]
    #[test]
    fn rejects_drive_relative_queries() {
        let temp = test_support::temp_dir("resolve-drive-relative");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let err = resolver
            .resolve(ResolveQuery {
                raw: "C:work",
                cwd: temp.path(),
            })
            .expect_err("drive-relative paths are unsupported");
        assert!(matches!(err, ResolveError::DriveRelativePath(_)));
    }

    #[cfg(unix)]
    #[test]
    fn propagates_non_not_found_direct_filesystem_errors() {
        let temp = test_support::temp_dir("resolve-filesystem-error");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let err = resolver
            .resolve(ResolveQuery {
                raw: "/dev/null/child",
                cwd: temp.path(),
            })
            .expect_err("non-directory traversal must fail explicitly");
        assert!(matches!(err, ResolveError::Filesystem { .. }));
    }

    #[test]
    fn errors_on_nonexistent_path() {
        let temp = test_support::temp_dir("resolve-miss");
        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);

        let query = ResolveQuery {
            raw: "./does-not-exist",
            cwd: temp.path(),
        };

        let err = resolver.resolve(query).expect_err("should error");
        assert!(matches!(err, ResolveError::NotFound));
    }

    #[test]
    fn resolve_leading_slash_direct_miss_falls_back_from_filesystem_root() {
        let temp = test_support::temp_dir("resolve-leading-slash-root");
        let canonical_temp = fs::canonicalize(temp.path()).expect("canonical temp dir");
        let missing_prefix = format!("dx-root-only-{}", std::process::id());
        let target = canonical_temp.join(&missing_prefix).join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query_string = format!("{}/{}{}", canonical_temp.display(), missing_prefix, "/pro");
        let query = ResolveQuery {
            raw: &query_string,
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("fallback should resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn resolve_leading_slash_direct_miss_does_not_use_bookmark_lookup() {
        let temp = test_support::temp_dir("resolve-leading-slash-no-bookmark");
        let missing_prefix = format!("dx-bookmark-only-{}", std::process::id());
        let resolver =
            Resolver::with_bookmark_lookup(AppConfig::default(), |_| Some(PathBuf::from("/tmp")));

        let query_string = format!("/{missing_prefix}/pro");
        let query = ResolveQuery {
            raw: &query_string,
            cwd: temp.path(),
        };

        let err = resolver
            .resolve(query)
            .expect_err("leading slash fallback should skip bookmarks");
        assert!(matches!(err, ResolveError::NotFound));
    }

    #[test]
    fn resolve_dot_slash_direct_miss_falls_back_to_abbreviation() {
        let temp = test_support::temp_dir("resolve-dot-slash-fallback");
        let root = temp.path().join("root");
        let target = root.join("no-local-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "./no-local-hit/pro",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("fallback should resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn resolve_tilde_slash_direct_miss_falls_back_to_abbreviation() {
        let temp = test_support::temp_dir("resolve-tilde-slash-fallback");
        let mut process = test_support::ScopedProcess::new();
        let home = temp.path().join("home");
        fs::create_dir_all(&home).expect("create home");

        let root = temp.path().join("root");
        let target = root.join("no-home-hit").join("project");
        fs::create_dir_all(&target).expect("create fallback target");

        process.set("HOME", &home);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![root]);
        let query = ResolveQuery {
            raw: "~/no-home-hit/pro",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("fallback should resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn resolve_prefixed_empty_fallback_query_preserves_path_not_found() {
        let temp = test_support::temp_dir("resolve-empty-prefixed-fallback");
        let mut process = test_support::ScopedProcess::new();
        let missing_home = temp.path().join("missing-home");

        process.set("HOME", &missing_home);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "~/",
            cwd: temp.path(),
        };

        let err = resolver
            .resolve(query)
            .expect_err("missing home directory should keep path-not-found");
        assert!(matches!(err, ResolveError::PathNotFound(_)));
    }

    #[test]
    fn returns_ambiguous_error_for_multiple_candidates() {
        let temp = test_support::temp_dir("resolve-ambiguous");
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("proj/alpha")).expect("create proj alpha");
        fs::create_dir_all(root.join("prod/alpha")).expect("create prod alpha");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "pro/al",
            cwd: temp.path(),
        };

        let err = resolver.resolve(query).expect_err("should be ambiguous");
        assert!(matches!(
            err,
            ResolveError::Ambiguous { candidates } if candidates.len() == 2
        ));
    }

    #[test]
    fn resolves_delimiter_aware_query() {
        let temp = test_support::temp_dir("resolve-delimiter-aware");
        let root = temp.path().join("root");
        let target = root.join("cd-extras");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "cd-e",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("delimiter-aware resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn resolves_doubled_period_query() {
        let temp = test_support::temp_dir("resolve-gap-aware");
        let root = temp.path().join("root");
        let target = root.join("PowerShell");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots_and_case_sensitivity(vec![root], false);
        let query = ResolveQuery {
            raw: "p..shell",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("gap-aware resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn resolves_multi_segment_delimiter_aware_query() {
        let temp = test_support::temp_dir("resolve-multi-delimiter-aware");
        let root = temp.path().join("root");
        let target = root.join("project/PowerShell/src/Microsoft.PowerShell.SDK");
        fs::create_dir_all(&target).expect("create target");

        let resolver = create_resolver_with_roots_and_case_sensitivity(vec![root], false);
        let query = ResolveQuery {
            raw: "pro/p..shell/s/.sdk",
            cwd: temp.path(),
        };

        let result = resolver
            .resolve(query)
            .expect("multi-segment delimiter-aware resolve");
        assert_eq!(result.path, target);
    }

    #[test]
    fn returns_ambiguous_error_for_delimiter_aware_candidates() {
        let temp = test_support::temp_dir("resolve-delimiter-ambiguous");
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("cd-extras")).expect("create cd-extras");
        fs::create_dir_all(root.join("cd-editor")).expect("create cd-editor");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "cd-e",
            cwd: temp.path(),
        };

        let err = resolver
            .resolve(query)
            .expect_err("delimiter-aware query should be ambiguous");
        assert!(matches!(
            err,
            ResolveError::Ambiguous { candidates } if candidates.len() == 2
        ));
    }

    #[test]
    fn step_up_alias_keeps_precedence_over_gap_syntax() {
        let temp = test_support::temp_dir("resolve-gap-precedence");
        let root = temp.path().join("root");
        fs::create_dir_all(root.join("..."))
            .expect("create literal triple-dot directory inside search root");

        let cwd = temp.path().join("a/b/c");
        fs::create_dir_all(&cwd).expect("create cwd");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "...",
            cwd: &cwd,
        };

        let result = resolver
            .resolve(query)
            .expect("step-up precedence should win");
        assert_eq!(result.path, temp.path().join("a"));
    }

    #[test]
    fn direct_resolution_wins_over_fallback_search_root() {
        let temp = test_support::temp_dir("resolve-precedence");
        let local = temp.path().join("src");
        fs::create_dir_all(&local).expect("create local src");

        let root = temp.path().join("root");
        fs::create_dir_all(root.join("src")).expect("create fallback src");

        let resolver = create_resolver_with_roots(vec![root]);
        let query = ResolveQuery {
            raw: "src",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("should resolve local");
        assert_eq!(result.path, local);
    }

    #[test]
    fn bookmark_resolves_when_no_filesystem_match_exists() {
        let temp = test_support::temp_dir("resolve-bookmark");
        let mut process = test_support::ScopedProcess::new();
        let bookmarks_file = temp.path().join("bookmarks.toml");
        let bookmark_target = temp.path().join("target");
        fs::create_dir_all(&bookmark_target).expect("create bookmark target");

        let canonical_target = fs::canonicalize(&bookmark_target).expect("canonical target");
        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            canonical_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        process.set("DX_BOOKMARKS_FILE", &bookmarks_file);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("bookmark should resolve");
        assert_eq!(result.path, canonical_target);
    }

    #[test]
    fn fallback_root_takes_precedence_over_bookmark() {
        let temp = test_support::temp_dir("resolve-fallback-over-bookmark");
        let mut process = test_support::ScopedProcess::new();
        let bookmarks_file = temp.path().join("bookmarks.toml");

        let fallback_root = temp.path().join("root");
        let fallback_match = fallback_root.join("proj");
        fs::create_dir_all(&fallback_match).expect("create fallback match");

        let bookmark_target = temp.path().join("bookmark-target");
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
        process.set("DX_BOOKMARKS_FILE", &bookmarks_file);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![fallback_root]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: temp.path(),
        };

        let result = resolver.resolve(query).expect("fallback should resolve");
        assert_eq!(result.path, fallback_match);
    }

    #[test]
    fn stale_bookmark_returns_no_match_and_resolution_fails() {
        let temp = test_support::temp_dir("resolve-stale-bookmark");
        let mut process = test_support::ScopedProcess::new();
        let bookmarks_file = temp.path().join("bookmarks.toml");
        let missing_target = temp.path().join("missing-target");

        let toml = format!(
            "[bookmarks]\nproj = \"{}\"\n",
            missing_target.display().to_string().replace('\\', "\\\\")
        );
        fs::write(&bookmarks_file, toml).expect("write bookmarks file");
        process.set("DX_BOOKMARKS_FILE", &bookmarks_file);

        let resolver = create_resolver_with_roots_and_bookmarks(vec![]);
        let query = ResolveQuery {
            raw: "proj",
            cwd: temp.path(),
        };

        let err = resolver
            .resolve(query)
            .expect_err("stale bookmark should fail");
        assert!(matches!(err, ResolveError::NotFound));
    }

    #[test]
    fn effective_roots_include_cwd_when_no_roots_configured() {
        let temp = test_support::temp_dir("effective-roots-cwd");
        let roots = super::super::build_effective_roots(temp.path(), &[]);
        assert_eq!(roots, vec![temp.path().to_path_buf()]);
    }

    #[test]
    fn effective_roots_dedup_when_cwd_already_configured() {
        let temp = test_support::temp_dir("effective-roots-dedup");
        let path = temp.path().to_path_buf();
        let roots = super::super::build_effective_roots(&path, std::slice::from_ref(&path));
        assert_eq!(roots, vec![path]);
    }
}
