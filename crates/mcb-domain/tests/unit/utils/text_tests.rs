//! Unit tests for `mcb_domain::utils::text` extraction utilities.

use rstest::rstest;
use serde::Serialize;
use serde_json::json;

use mcb_domain::utils::text::{extract_text, extract_text_from, extract_text_with_sep};

// ---------------------------------------------------------------------------
// extract_text_with_sep
// ---------------------------------------------------------------------------

#[rstest]
fn extract_text_with_sep_single_item() {
    let content = vec![json!({"text": "hello"})];
    assert_eq!(extract_text_with_sep(&content, ", "), "hello");
}

#[rstest]
fn extract_text_with_sep_multiple_items() {
    let content = vec![
        json!({"text": "hello"}),
        json!({"text": "world"}),
        json!({"text": "foo"}),
    ];
    assert_eq!(extract_text_with_sep(&content, ", "), "hello, world, foo");
}

#[rstest]
fn extract_text_with_sep_empty_separator() {
    let content = vec![json!({"text": "a"}), json!({"text": "b"})];
    assert_eq!(extract_text_with_sep(&content, ""), "ab");
}

#[rstest]
fn extract_text_with_sep_empty_content() {
    let content: Vec<serde_json::Value> = vec![];
    assert_eq!(extract_text_with_sep(&content, ", "), "");
}

#[rstest]
fn extract_text_with_sep_skips_non_text_entries() {
    let content = vec![
        json!({"text": "kept"}),
        json!({"type": "image", "url": "http://example.com/img.png"}),
        json!({"text": "also kept"}),
        json!(42),
        json!(null),
        json!({"resource": {"uri": "file:///foo"}}),
    ];
    assert_eq!(extract_text_with_sep(&content, " | "), "kept | also kept");
}

#[rstest]
fn extract_text_with_sep_skips_non_string_text_field() {
    // A "text" field that is not a string should be skipped
    let content = vec![
        json!({"text": 123}),
        json!({"text": true}),
        json!({"text": null}),
        json!({"text": "valid"}),
    ];
    assert_eq!(extract_text_with_sep(&content, ", "), "valid");
}

// ---------------------------------------------------------------------------
// extract_text (newline separator)
// ---------------------------------------------------------------------------

#[rstest]
fn extract_text_joins_with_newline() {
    let content = vec![json!({"text": "line 1"}), json!({"text": "line 2"})];
    assert_eq!(extract_text(&content), "line 1\nline 2");
}

#[rstest]
fn extract_text_empty() {
    let content: Vec<serde_json::Value> = vec![];
    assert_eq!(extract_text(&content), "");
}

// ---------------------------------------------------------------------------
// extract_text_from (generic serializable types)
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
struct FakeContent {
    text: String,
}

#[derive(Debug, Serialize)]
struct ImageContent {
    r#type: String,
    url: String,
}

/// A type that deliberately fails serialization.
struct UnserializableContent;

impl Serialize for UnserializableContent {
    fn serialize<S: serde::Serializer>(&self, _serializer: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("intentional failure"))
    }
}

#[rstest]
fn extract_text_from_with_struct() {
    let content = vec![
        FakeContent {
            text: "hello".to_owned(),
        },
        FakeContent {
            text: "world".to_owned(),
        },
    ];
    assert_eq!(extract_text_from(&content), "hello\nworld");
}

#[rstest]
fn extract_text_from_skips_non_text_structs() {
    // ImageContent has no "text" field -> should be skipped
    let content = vec![ImageContent {
        r#type: "image".to_owned(),
        url: "http://example.com".to_owned(),
    }];
    assert_eq!(extract_text_from(&content), "");
}

#[rstest]
fn extract_text_from_skips_serialization_failures() {
    let content = vec![UnserializableContent, UnserializableContent];
    // filter_map(|c| serde_json::to_value(c).ok()) should skip these
    assert_eq!(extract_text_from(&content), "");
}

#[rstest]
fn extract_text_from_empty() {
    let content: Vec<FakeContent> = vec![];
    assert_eq!(extract_text_from(&content), "");
}

#[rstest]
fn extract_text_from_mixed_valid_and_invalid() {
    // Can't mix types in a Vec easily, so use serde_json::Value as the generic type
    // to simulate the case where some values have "text" and some don't
    let content = vec![
        json!({"text": "first"}),
        json!({"no_text_here": true}),
        json!({"text": "second"}),
    ];
    assert_eq!(extract_text_from(&content), "first\nsecond");
}
