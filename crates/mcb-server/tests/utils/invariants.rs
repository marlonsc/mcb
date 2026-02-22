use rmcp::model::CallToolResult;

pub fn error_text(result: &CallToolResult) -> String {
    serde_json::to_value(&result.content)
        .ok()
        .and_then(|value| value.as_array().cloned())
        .and_then(|items| items.first().cloned())
        .and_then(|item| item.get("text").cloned())
        .and_then(|text| text.as_str().map(ToOwned::to_owned))
        .unwrap_or_default()
}

pub fn assert_error_shape(result: &CallToolResult, expected_message: &str) {
    assert_eq!(result.is_error, Some(true), "expected is_error=true");

    let content_json_result = serde_json::to_value(&result.content);
    assert!(content_json_result.is_ok(), "serialize content");
    let content_json = match content_json_result {
        Ok(value) => value,
        Err(_) => return,
    };
    assert!(content_json.is_array(), "error content must be an array");
    assert!(
        content_json
            .as_array()
            .is_some_and(|items| items.first().and_then(|item| item.get("text")).is_some()),
        "error content must contain a text field"
    );

    let text = error_text(result);
    assert!(
        text.contains(expected_message),
        "expected '{expected_message}' in '{text}'"
    );
}

pub fn assert_invalid_params(error: &rmcp::ErrorData, expected_substring: &str) {
    assert_eq!(
        error.code.0, -32602,
        "expected error code -32602, got {}",
        error.code.0
    );
    assert!(
        error.message.contains(expected_substring),
        "expected '{}' in error message: {}",
        expected_substring,
        error.message
    );
}

#[cfg(test)]
mod tests {
    use super::{assert_error_shape, assert_invalid_params, error_text};
    use rmcp::model::CallToolResult;

    #[test]
    fn invariants_helpers_are_linked() {
        let _ = error_text as fn(&CallToolResult) -> String;
        let _ = assert_error_shape as fn(&CallToolResult, &str);
        let _ = assert_invalid_params as fn(&rmcp::ErrorData, &str);
    }
}
