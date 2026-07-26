//! Non-Unix placeholder: reports cancellation so the shell falls back to its
//! own completion rather than losing the keystroke.

use super::{MenuOptions, MenuRequest, MenuResult};

pub fn select(request: MenuRequest<'_>, _options: &MenuOptions) -> Option<MenuResult> {
    Some(MenuResult::Cancelled {
        filter_query: request.query.to_string(),
        changed_query: false,
        geometry: None,
    })
}
