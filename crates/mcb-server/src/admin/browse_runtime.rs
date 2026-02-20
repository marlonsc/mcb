//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use rmcp::model::CallToolRequestParams;
use serde::de::DeserializeOwned;

use crate::admin::handlers::AdminState;
use crate::tools::{ToolExecutionContext, route_tool_call};
use crate::utils::text::extract_text;

pub(super) async fn execute_tool_json<T: DeserializeOwned>(
    state: &AdminState,
    tool_name: &str,
    args: serde_json::Value,
) -> Result<T, String> {
    let handlers = state
        .tool_handlers
        .as_ref()
        .ok_or_else(|| "Unified execution handlers are not available".to_owned())?;

    let arguments = args
        .as_object()
        .cloned()
        .ok_or_else(|| format!("{tool_name} arguments must be a JSON object"))?;

    let request = CallToolRequestParams {
        name: tool_name.to_owned().into(),
        arguments: Some(arguments),
        task: None,
        meta: None,
    };

    let result = route_tool_call(request, handlers, ToolExecutionContext::default())
        .await
        .map_err(|e| e.message.to_string())?;

    let text = extract_text(&result.content);
    if result.is_error.unwrap_or(false) {
        return Err(if text.is_empty() {
            format!("{tool_name} execution failed")
        } else {
            text
        });
    }

    serde_json::from_str(&text).map_err(|e| format!("Failed to parse {tool_name} output JSON: {e}"))
}
