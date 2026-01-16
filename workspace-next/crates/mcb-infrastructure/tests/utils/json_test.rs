//! JSON Utility Tests

use mcb_infrastructure::utils::JsonExt;
use serde_json::json;
use std::collections::HashMap;

#[test]
fn test_json_value_ext() {
    let value = json!({
        "string": "hello",
        "number": 42,
        "float": 3.14,
        "bool": true
    });

    assert_eq!(value.str_or("string", "default"), "hello");
    assert_eq!(value.str_or("missing", "default"), "default");
    assert_eq!(value.i64_or("number", 0), 42);
    assert_eq!(value.f64_or("float", 0.0), 3.14);
    assert!(value.bool_or("bool", false));
}

#[test]
fn test_hashmap_ext() {
    let mut map = HashMap::new();
    map.insert("key".to_string(), json!("value"));
    map.insert("num".to_string(), json!(123));

    assert_eq!(map.str_or("key", "default"), "value");
    assert_eq!(map.i64_or("num", 0), 123);
    assert_eq!(map.str_or("missing", "default"), "default");
}

#[test]
fn test_optional_methods() {
    let value = json!({"present": "value"});

    assert_eq!(value.opt_str("present"), Some("value"));
    assert_eq!(value.opt_str("missing"), None);
}
