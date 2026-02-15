/// Resolves typed values from MCP request and context metadata.
///
/// This module extracts session IDs, project IDs, and other context values
/// from the `Meta` maps attached to tool call requests, handling multiple
/// key name conventions (`snake_case`, camelCase, x-prefixed).
use rmcp::model::Meta;
use serde_json::Value;

fn meta_value_as_string(meta: &Meta, keys: &[&str]) -> Option<String> {
    for key in keys {
        let value = meta.get(*key)?;
        let extracted = match value {
            Value::String(v) => Some(v.clone()),
            Value::Number(v) => Some(v.to_string()),
            Value::Bool(v) => Some(v.to_string()),
            _ => None,
        };
        if extracted.is_some() {
            return extracted;
        }
    }
    None
}

pub(crate) fn resolve_context_value(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<String> {
    request_meta
        .and_then(|meta| meta_value_as_string(meta, keys))
        .or_else(|| meta_value_as_string(context_meta, keys))
}

fn meta_value_as_bool(meta: &Meta, keys: &[&str]) -> Option<bool> {
    for key in keys {
        let value = meta.get(*key)?;
        let extracted = match value {
            Value::Bool(v) => Some(*v),
            Value::String(v) => match v.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        };
        if extracted.is_some() {
            return extracted;
        }
    }
    None
}

pub(crate) fn resolve_context_bool(
    request_meta: Option<&Meta>,
    context_meta: &Meta,
    keys: &[&str],
) -> Option<bool> {
    request_meta
        .and_then(|meta| meta_value_as_bool(meta, keys))
        .or_else(|| meta_value_as_bool(context_meta, keys))
}
