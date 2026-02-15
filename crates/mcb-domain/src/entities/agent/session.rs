use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::types::{AgentSessionStatus, AgentType};

/// Represents an agent session with execution metadata and results.
///
/// Tracks the lifecycle of an agent execution, including timing, status, resource usage,
/// and summaries of the prompt and results.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentSession {
    /// Unique identifier for this agent session.
    pub id: String,
    /// Reference to the associated session summary.
    pub session_summary_id: String,
    /// Type of agent that executed this session.
    pub agent_type: AgentType,
    /// Model used by the agent (e.g., "claude-3-opus").
    pub model: String,
    /// Optional parent session ID if this is a nested/delegated session.
    pub parent_session_id: Option<String>,
    /// Unix timestamp (milliseconds) when the session started.
    pub started_at: i64,
    /// Unix timestamp (milliseconds) when the session ended, if completed.
    pub ended_at: Option<i64>,
    /// Total duration of the session in milliseconds.
    pub duration_ms: Option<i64>,
    /// Current status of the session (running, completed, failed, etc.).
    pub status: AgentSessionStatus,
    /// Brief summary of the input prompt.
    pub prompt_summary: Option<String>,
    /// Brief summary of the execution result.
    pub result_summary: Option<String>,
    /// Total tokens consumed during this session.
    pub token_count: Option<i64>,
    /// Number of tool calls made during this session.
    pub tool_calls_count: Option<i64>,
    /// Number of delegations made during this session.
    pub delegations_count: Option<i64>,
    /// Optional project ID this session belongs to.
    pub project_id: Option<String>,
    /// Optional worktree ID this session is working in.
    pub worktree_id: Option<String>,
}
