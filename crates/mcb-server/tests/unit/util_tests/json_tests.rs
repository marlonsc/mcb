use mcb_server::utils::json::json_map;
use rstest::rstest;
use serde_json::json;

#[rstest]
#[test]
fn test_json_map() {
    let val = Some(json!({"key": "value"}));
    let map = json_map(&val).unwrap();
    assert_eq!(map.get("key").unwrap().as_str().unwrap(), "value");
}

#[rstest]
#[test]
fn test_get_str() {
    let val = json!({"key": "value"});
    let map = val.as_object().unwrap();
    assert_eq!(
        map.get("key").and_then(|value| value.as_str()),
        Some("value")
    );
    assert_eq!(map.get("missing").and_then(|value| value.as_str()), None);
}

#[rstest]
#[case(json!({"key": 42}), Some(42))]
#[case(json!({"missing": 10}), None)]
fn test_get_i64(#[case] value: serde_json::Value, #[case] expected: Option<i64>) {
    let map = value.as_object().unwrap();
    assert_eq!(map.get("key").and_then(serde_json::Value::as_i64), expected);
}

#[rstest]
#[case(json!({"key": true}), Some(true))]
#[case(json!({"missing": false}), None)]
fn test_get_bool(#[case] value: serde_json::Value, #[case] expected: Option<bool>) {
    let map = value.as_object().unwrap();
    assert_eq!(
        map.get("key").and_then(serde_json::Value::as_bool),
        expected
    );
}

#[rstest]
#[test]
fn test_get_string_list() {
    let val = json!({"key": ["a", "b", "c"]});
    let map = val.as_object().unwrap();
    let values = map
        .get("key")
        .and_then(|entry| entry.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_owned))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(values, vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]);
}
