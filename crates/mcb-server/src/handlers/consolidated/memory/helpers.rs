use mcb_domain::entities::memory::{ExecutionType, ObservationType, QualityGateStatus};
use rmcp::model::{CallToolResult, Content};
use serde_json::{Map, Value};

pub struct MemoryHelpers;

impl MemoryHelpers {
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

    pub fn get_i32(data: &Map<String, Value>, key: &str) -> Option<i32> {
        data.get(key)
            .and_then(|value| value.as_i64())
            .and_then(|v| v.try_into().ok())
    }

    pub fn get_f32(data: &Map<String, Value>, key: &str) -> Option<f32> {
        data.get(key)
            .and_then(|value| value.as_f64())
            .map(|v| v as f32)
    }

    pub fn get_bool(data: &Map<String, Value>, key: &str) -> Option<bool> {
        data.get(key).and_then(|value| value.as_bool())
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

    pub fn parse_observation_type(value: &str) -> Result<ObservationType, CallToolResult> {
        match value.to_lowercase().as_str() {
            "code" => Ok(ObservationType::Code),
            "decision" => Ok(ObservationType::Decision),
            "context" => Ok(ObservationType::Context),
            "error" => Ok(ObservationType::Error),
            "summary" => Ok(ObservationType::Summary),
            "execution" => Ok(ObservationType::Execution),
            "quality_gate" => Ok(ObservationType::QualityGate),
            _ => Err(CallToolResult::error(vec![Content::text(format!(
                "Unknown observation type: {value}"
            ))])),
        }
    }

    pub fn parse_execution_type(value: &str) -> Result<ExecutionType, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown execution type: {value}"
            ))])
        })
    }

    pub fn parse_quality_gate_status(value: &str) -> Result<QualityGateStatus, CallToolResult> {
        value.parse().map_err(|_| {
            CallToolResult::error(vec![Content::text(format!(
                "Unknown quality gate status: {value}"
            ))])
        })
    }
}
