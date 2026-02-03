//! Agent session tracking entities.

use serde::{Deserialize, Serialize};

/// Session for a single agent execution.
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

/// Delegation event between agent sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub id: String,
    pub parent_session_id: String,
    pub child_session_id: String,
    pub prompt: String,
    pub prompt_embedding_id: Option<String>,
    pub result: Option<String>,
    pub success: bool,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub duration_ms: Option<i64>,
}

/// Tool invocation within an agent session.
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

/// Checkpoint snapshot captured during an agent session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub id: String,
    pub session_id: String,
    pub checkpoint_type: CheckpointType,
    pub description: String,
    pub snapshot_data: serde_json::Value,
    pub created_at: i64,
    pub restored_at: Option<i64>,
    pub expired: bool,
}

/// Agent role for the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentType {
    Sisyphus,
    Oracle,
    Explore,
}

impl AgentType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sisyphus => "sisyphus",
            Self::Oracle => "oracle",
            Self::Explore => "explore",
        }
    }
}

impl std::str::FromStr for AgentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sisyphus" => Ok(Self::Sisyphus),
            "oracle" => Ok(Self::Oracle),
            "explore" => Ok(Self::Explore),
            _ => Err(format!("Unknown agent type: {s}")),
        }
    }
}

/// Agent session status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentSessionStatus {
    Active,
    Completed,
    Failed,
}

impl AgentSessionStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl std::str::FromStr for AgentSessionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(format!("Unknown agent session status: {s}")),
        }
    }
}

/// Checkpoint type captured during a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckpointType {
    Git,
    File,
    Config,
}

impl CheckpointType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::File => "file",
            Self::Config => "config",
        }
    }
}

impl std::str::FromStr for CheckpointType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "git" => Ok(Self::Git),
            "file" => Ok(Self::File),
            "config" => Ok(Self::Config),
            _ => Err(format!("Unknown checkpoint type: {s}")),
        }
    }
}
