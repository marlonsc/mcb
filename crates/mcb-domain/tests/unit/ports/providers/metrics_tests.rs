//! Unit tests for metrics macros
//!
//! Tests for `labels!` macro.

use mcb_domain::labels;
use mcb_domain::ports::MetricLabels;

#[test]
fn test_labels_macro() {
    let empty: MetricLabels = labels!();
    assert!(empty.is_empty());

    let with_values = labels!("collection" => "test", "provider" => "ollama");
    assert_eq!(with_values.len(), 2);
    assert_eq!(with_values.get("collection"), Some(&"test".to_owned()));
    assert_eq!(with_values.get("provider"), Some(&"ollama".to_owned()));
}
