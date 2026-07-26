//! `dx` resolves abbreviated directory queries to a single path, so `dx p/s/m`
//! reaches `projects/sample/module` without typing it out.
//!
//! The binary is a thin shell over this library: [`cli`] parses arguments and
//! maps errors to exit codes, and everything it prints is consumed by a shell
//! hook that `dx init` generates.
//!
//! Resolution is the core ([`resolve`]), reading its settings from [`config`]
//! and its candidates from [`stacks`], bookmarks and a zoxide database when one
//! exists. [`complete`] exposes the same candidates to shell completion, and
//! [`menu`] renders them interactively.
//!
//! Only the modules linked above are public. Bookmarks, hook generation,
//! frecency and shared file helpers are crate-internal: the binary reaches them
//! through `cli`, and nothing outside this crate should depend on their shape.
//!
//! # Example
//!
//! ```no_run
//! use dx::config::AppConfig;
//! use dx::resolve::{ResolveQuery, Resolver};
//!
//! let resolver = Resolver::from_environment()?;
//! let resolved = resolver.resolve(ResolveQuery {
//!     raw: "p/s",
//!     cwd: std::path::Path::new("."),
//! })?;
//! println!("{}", resolved.path.display());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub(crate) mod bookmarks;
pub mod cli;
pub(crate) mod common;
pub mod complete;
pub mod config;
pub(crate) mod frecency;
pub(crate) mod hooks;
pub mod menu;
pub mod resolve;
pub mod stacks;
#[cfg(test)]
pub(crate) mod test_support;
