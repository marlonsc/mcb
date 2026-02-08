use mcb_server::utils::json::*;
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
    assert_eq!(get_str(map, "key"), Some("value".to_string()));
    assert_eq!(get_str(map, "missing"), None);
}

#[test]
fn test_get_i64() {
    let val = json!({"key": 42});
    let map = val.as_object().unwrap();
    assert_eq!(get_i64(map, "key"), Some(42));
}

#[test]
fn test_get_bool() {
    let val = json!({"key": true});
    let map = val.as_object().unwrap();
    assert_eq!(get_bool(map, "key"), Some(true));
}

#[test]
fn test_get_string_list() {
    let val = json!({"key": ["a", "b", "c"]});
    let map = val.as_object().unwrap();
    assert_eq!(
        get_string_list(map, "key"),
        vec!["a".to_string(), "b".to_string(), "c".to_string()]
    );
}
