//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! JSON utilities for MCP server tool handlers.
//!
use serde_json::{Map, Value};

/// Extracts a JSON object map from an optional JSON value.
///
/// Returns a reference to the underlying map if the value is an object, or `None` otherwise.
#[must_use]
pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
    data.as_ref().and_then(|value| value.as_object())
}

/// Convert a JSON value to a TOML value
pub fn json_to_toml(json: &serde_json::Value) -> Option<toml::Value> {
    match json {
        serde_json::Value::Null => Some(toml::Value::String(String::new())),
        serde_json::Value::Bool(b) => Some(toml::Value::Boolean(*b)),
        serde_json::Value::Number(n) => n
            .as_i64()
            .map(toml::Value::Integer)
            .or_else(|| n.as_f64().map(toml::Value::Float)),
        serde_json::Value::String(s) => Some(toml::Value::String(s.clone())),
        serde_json::Value::Array(arr) => {
            let toml_arr: Option<Vec<toml::Value>> = arr.iter().map(json_to_toml).collect();
            toml_arr.map(toml::Value::Array)
        }
        serde_json::Value::Object(obj) => {
            let mut table = toml::map::Map::new();
            for (k, v) in obj {
                table.insert(k.clone(), json_to_toml(v)?);
            }
            Some(toml::Value::Table(table))
        }
    }
}
