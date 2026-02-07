use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

pub struct SessionHelpers;

impl SessionHelpers {
    pub fn json_map(data: &Option<Value>) -> Option<&Map<String, Value>> {
        data.as_ref().and_then(|value| value.as_object())
    }

    pub fn get_str(data: &Map<String, Value>, key: &str) -> Option<String> {
        data.get(key)
            .and_then(|value| value.as_str())
            .map(str::to_string)
    }

    pub fn get_i64(data: &Map<String, Value>, key: &str) -> Option<i64> {
        data.get(key).and_then(|value| value.as_i64())
    }

    pub fn get_string_list(data: &Map<String, Value>, key: &str) -> Vec<String> {
        data.get(key)
            .and_then(|value| value.as_array())
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_required_str(
        data: &Map<String, Value>,
        key: &str,
    ) -> Result<String, CallToolResult> {
        Self::get_str(data, key).ok_or_else(|| {
            CallToolResult::error(vec![Content::text(format!(
                "Missing required field: {key}"
            ))])
        })
    }

    pub fn parse_agent_type(value: &str) -> Result<AgentType, McpError> {
        value
            .parse()
            .map_err(|_| McpError::invalid_params("Invalid agent_type", None))
    }

    pub fn parse_status(value: &str) -> Result<AgentSessionStatus, McpError> {
        value
            .parse()
            .map_err(|_| McpError::invalid_params("Invalid status", None))
    }
}
