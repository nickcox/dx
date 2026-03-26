use crate::resolve::{ResolveMode, Resolver};

pub fn run_resolve(resolver: &Resolver, query: &str, list: bool, json: bool) -> i32 {
    let mode = if json {
        ResolveMode::Json
    } else if list {
        ResolveMode::List
    } else {
        ResolveMode::Default
    };

    resolver.execute(query, mode)
}
