// Re-export from production code — single source of truth
pub use mcb_server::utils::text::{extract_text, extract_text_with_sep};

pub fn parse_json_text(text: &str) -> Option<serde_json::Value> {
    serde_json::from_str(text).ok()
}

pub fn parse_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, serde_json::Error> {
    serde_json::from_str(text)
}

pub fn parse_count_from_json_text(text: &str) -> usize {
    parse_json_text(text)
        .and_then(|v| v.get("count").and_then(serde_json::Value::as_u64))
        .map_or(0, |v| v as usize)
}

#[cfg(test)]
mod tests {
    use super::{extract_text, parse_count_from_json_text, parse_json, parse_json_text};

    #[test]
    fn parse_json_text_and_count_work() {
        let value_opt = parse_json_text(r#"{"count":3}"#);
        assert!(value_opt.is_some(), "json");
        let value = match value_opt {
            Some(value) => value,
            None => return,
        };
        assert_eq!(
            value.get("count").and_then(serde_json::Value::as_u64),
            Some(3)
        );
        assert_eq!(parse_count_from_json_text(r#"{"count":7}"#), 7);
        assert_eq!(parse_count_from_json_text("not-json"), 0);
    }

    #[test]
    fn extract_text_handles_empty_slice() {
        let content: [rmcp::model::Content; 0] = [];
        assert!(extract_text(&content).is_empty());
    }

    #[test]
    fn parse_json_works_for_typed_values() {
        let parsed = parse_json::<serde_json::Value>(r#"{"ok":true}"#);
        assert!(parsed.is_ok(), "value parse failed");
        let value = match parsed {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(
            value.get("ok").and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }
}
