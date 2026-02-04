use serde::{Deserialize, Serialize};

use super::types::{AgentSessionStatus, AgentType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub session_summary_id: String,
    pub agent_type: AgentType,
    pub model: String,
    pub parent_session_id: Option<String>,
    pub started_at: i64,
    pub ended_at: Option<i64>,
    pub duration_ms: Option<i64>,
    pub status: AgentSessionStatus,
    pub prompt_summary: Option<String>,
    pub result_summary: Option<String>,
    pub token_count: Option<i64>,
    pub tool_calls_count: Option<i64>,
    pub delegations_count: Option<i64>,
}
