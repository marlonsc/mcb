//! Test for serde Deserialize form handling.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct TestFilterParams {
    /// Full-text search term (renamed from query param "q")
    #[serde(rename = "q", default)]
    pub search: Option<String>,

    /// Column to sort by (renamed from query param "sort")
    #[serde(rename = "sort", default)]
    pub sort_field: Option<String>,

    /// Sort direction (renamed from query param "order")
    #[serde(rename = "order", default)]
    pub sort_order: Option<String>,

    /// 1-based page number (default 1)
    #[serde(default = "default_page")]
    pub page: usize,

    /// Records per page (default 20)
    #[serde(default = "default_per_page")]
    pub per_page: usize,
}

fn default_page() -> usize {
    1
}

fn default_per_page() -> usize {
    20
}

#[test]
fn test_filter_params_deserialize_from_form_data() {
    // Simulate form data as would come from a POST request
    let form_data = serde_json::json!({
        "q": "search term",
        "sort": "name",
        "order": "desc",
        "page": 2,
        "per_page": 50,
    });

    let params: TestFilterParams =
        serde_json::from_value(form_data).expect("Failed to deserialize form data");

    assert_eq!(params.search, Some("search term".to_owned()));
    assert_eq!(params.sort_field, Some("name".to_owned()));
    assert_eq!(params.sort_order, Some("desc".to_owned()));
    assert_eq!(params.page, 2);
    assert_eq!(params.per_page, 50);
}

#[test]
fn test_filter_params_deserialize_with_defaults() {
    // Test that defaults are applied when fields are missing
    let form_data = serde_json::json!({
        "q": "search",
    });

    let params: TestFilterParams =
        serde_json::from_value(form_data).expect("Failed to deserialize form data with defaults");

    assert_eq!(params.search, Some("search".to_owned()));
    assert_eq!(params.sort_field, None);
    assert_eq!(params.sort_order, None);
    assert_eq!(params.page, 1);
    assert_eq!(params.per_page, 20);
}

#[test]
fn test_filter_params_deserialize_empty() {
    // Test that all defaults are applied when form is empty
    let form_data = serde_json::json!({});

    let params: TestFilterParams =
        serde_json::from_value(form_data).expect("Failed to deserialize empty form data");

    assert_eq!(params.search, None);
    assert_eq!(params.sort_field, None);
    assert_eq!(params.sort_order, None);
    assert_eq!(params.page, 1);
    assert_eq!(params.per_page, 20);
}
