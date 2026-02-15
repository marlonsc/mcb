pub fn extract_text(content: &[rmcp::model::Content]) -> String {
    extract_text_with_sep(content, "\n")
}

pub fn extract_text_with_sep(content: &[rmcp::model::Content], sep: &str) -> String {
    content
        .iter()
        .filter_map(|c| {
            if let Ok(json) = serde_json::to_value(c)
                && let Some(text) = json.get("text")
            {
                text.as_str().map(str::to_string)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join(sep)
}

pub fn parse_json_text(text: &str) -> Option<serde_json::Value> {
    serde_json::from_str(text).ok()
}

pub fn parse_json<T: serde::de::DeserializeOwned>(text: &str, context: &str) -> T {
    serde_json::from_str(text).unwrap_or_else(|error| {
        panic!("{context}: {error}");
    })
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
        let value = parse_json_text(r#"{"count":3}"#).expect("json");
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
        let value = parse_json::<serde_json::Value>(r#"{"ok":true}"#, "value parse failed");
        assert_eq!(
            value.get("ok").and_then(serde_json::Value::as_bool),
            Some(true)
        );
    }
}
