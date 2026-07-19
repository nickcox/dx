use serde::Serialize;

/// The action returned by `dx menu` as JSON on stdout.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "action")]
pub enum MenuAction {
    /// Replace a byte range of the original buffer with the selected path.
    #[serde(rename = "replace")]
    Replace {
        #[serde(rename = "replaceStart")]
        replace_start: usize,
        #[serde(rename = "replaceEnd")]
        replace_end: usize,
        value: String,
        terminal: TerminalState,
        #[serde(flatten)]
        geometry: Option<TerminalGeometry>,
    },
    /// Explicit user cancellation after an interactive menu session.
    #[serde(rename = "cancel")]
    Cancel {
        #[serde(skip_serializing_if = "Option::is_none")]
        terminal: Option<TerminalState>,
        #[serde(flatten)]
        geometry: Option<TerminalGeometry>,
    },
    /// No operation — the buffer should remain unchanged.
    #[serde(rename = "noop")]
    Noop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TerminalState {
    Clean,
    Dirty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct TerminalGeometry {
    #[serde(rename = "redrawRow")]
    pub redraw_row: u16,
    #[serde(rename = "scrollRows")]
    pub scroll_rows: u16,
}

impl MenuAction {
    pub fn replace(
        replace_start: usize,
        replace_end: usize,
        value: String,
        terminal: TerminalState,
        geometry: Option<TerminalGeometry>,
    ) -> Self {
        MenuAction::Replace {
            replace_start,
            replace_end,
            value,
            terminal,
            geometry,
        }
    }

    pub fn noop() -> Self {
        MenuAction::Noop
    }

    pub fn cancel(geometry: Option<TerminalGeometry>) -> Self {
        MenuAction::Cancel {
            terminal: geometry.map(|_| TerminalState::Dirty),
            geometry,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).expect("MenuAction serialization cannot fail")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_serializes_correctly() {
        let action = MenuAction::noop();
        assert_eq!(action.to_json(), r#"{"action":"noop"}"#);
    }

    #[test]
    fn cancel_serializes_correctly() {
        let action = MenuAction::cancel(None);
        assert_eq!(action.to_json(), r#"{"action":"cancel"}"#);
    }

    #[test]
    fn replace_serializes_with_camel_case_fields() {
        let action = MenuAction::replace(
            3,
            6,
            "/home/user/bar".to_string(),
            TerminalState::Clean,
            None,
        );
        let json = action.to_json();
        assert!(json.contains(r#""action":"replace""#));
        assert!(json.contains(r#""replaceStart":3"#));
        assert!(json.contains(r#""replaceEnd":6"#));
        assert!(json.contains(r#""value":"/home/user/bar""#));
        assert!(json.contains(r#""terminal":"clean""#));
        assert!(!json.contains("redrawRow"));
        assert!(!json.contains("scrollRows"));
    }

    #[test]
    fn replace_roundtrips_through_json() {
        let action = MenuAction::replace(
            0,
            10,
            "/tmp".to_string(),
            TerminalState::Dirty,
            Some(TerminalGeometry {
                redraw_row: 13,
                scroll_rows: 10,
            }),
        );
        let json = action.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["action"], "replace");
        assert_eq!(parsed["replaceStart"], 0);
        assert_eq!(parsed["replaceEnd"], 10);
        assert_eq!(parsed["value"], "/tmp");
        assert_eq!(parsed["terminal"], "dirty");
        assert_eq!(parsed["redrawRow"], 13);
        assert_eq!(parsed["scrollRows"], 10);
    }

    #[test]
    fn dirty_cancel_serializes_terminal_geometry() {
        let action = MenuAction::cancel(Some(TerminalGeometry {
            redraw_row: 4,
            scroll_rows: 2,
        }));

        assert_eq!(
            action.to_json(),
            r#"{"action":"cancel","terminal":"dirty","redrawRow":4,"scrollRows":2}"#
        );
    }
}
