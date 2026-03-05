//! Schema helper types for MCP tool argument definitions.

/// Schema proxy for JSON object data payloads.
///
/// Generates `{"type": "object", "additionalProperties": {}}` in JSON Schema,
/// instead of the empty schema `{}` produced by `serde_json::Value`.
///
/// Use via `#[schemars(with = "ObjectDataSchema")]` on `Option<serde_json::Value>` fields.
pub type ObjectDataSchema = std::collections::HashMap<String, serde_json::Value>;
