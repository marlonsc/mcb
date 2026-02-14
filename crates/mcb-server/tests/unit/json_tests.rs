use mcb_server::utils::json::{JsonMapExt, json_map};
use rstest::rstest;
use serde_json::json;

#[test]
fn test_json_map() {
    let val = Some(json!({"key": "value"}));
    let map = json_map(&val).unwrap();
    assert_eq!(map.get("key").unwrap().as_str().unwrap(), "value");
}

#[test]
fn test_get_str() {
    let val = json!({"key": "value"});
    let map = val.as_object().unwrap();
    assert_eq!(map.string("key"), Some("value".to_string()));
    assert_eq!(map.string("missing"), None);
}

#[rstest]
#[case(json!({"key": 42}), Some(42))]
#[case(json!({"missing": 10}), None)]
#[test]
fn test_get_i64(#[case] value: serde_json::Value, #[case] expected: Option<i64>) {
    let map = value.as_object().unwrap();
    assert_eq!(map.int64("key"), expected);
}

#[rstest]
#[case(json!({"key": true}), Some(true))]
#[case(json!({"missing": false}), None)]
#[test]
fn test_get_bool(#[case] value: serde_json::Value, #[case] expected: Option<bool>) {
    let map = value.as_object().unwrap();
    assert_eq!(map.boolean("key"), expected);
}

#[test]
fn test_get_string_list() {
    let val = json!({"key": ["a", "b", "c"]});
    let map = val.as_object().unwrap();
    assert_eq!(
        map.string_list("key"),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}
