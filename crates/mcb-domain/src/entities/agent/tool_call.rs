use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub session_id: String,
    pub tool_name: String,
    pub params_summary: Option<String>,
    pub success: bool,
    pub error_message: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: i64,
}
