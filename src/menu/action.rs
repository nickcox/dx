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
    },
    /// No operation — the buffer should remain unchanged.
    #[serde(rename = "noop")]
    Noop,
}

impl MenuAction {
    pub fn replace(replace_start: usize, replace_end: usize, value: String) -> Self {
        MenuAction::Replace {
            replace_start,
            replace_end,
            value,
        }
    }

    pub fn noop() -> Self {
        MenuAction::Noop
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
    fn replace_serializes_with_camel_case_fields() {
        let action = MenuAction::replace(3, 6, "/home/user/bar".to_string());
        let json = action.to_json();
        assert!(json.contains(r#""action":"replace""#));
        assert!(json.contains(r#""replaceStart":3"#));
        assert!(json.contains(r#""replaceEnd":6"#));
        assert!(json.contains(r#""value":"/home/user/bar""#));
    }

    #[test]
    fn replace_roundtrips_through_json() {
        let action = MenuAction::replace(0, 10, "/tmp".to_string());
        let json = action.to_json();
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["action"], "replace");
        assert_eq!(parsed["replaceStart"], 0);
        assert_eq!(parsed["replaceEnd"], 10);
        assert_eq!(parsed["value"], "/tmp");
    }
}
