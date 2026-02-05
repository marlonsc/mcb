use rmcp::model::CallToolResult;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub type HookResult<T> = Result<T, HookError>;

#[derive(Debug, Clone)]
pub enum HookError {
    MemoryServiceUnavailable,
    FailedToStoreObservation(String),
    FailedToInjectContext(String),
    InvalidToolOutput(String),
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookError::MemoryServiceUnavailable => write!(f, "Memory service unavailable"),
            HookError::FailedToStoreObservation(msg) => {
                write!(f, "Failed to store observation: {}", msg)
            }
            HookError::FailedToInjectContext(msg) => write!(f, "Failed to inject context: {}", msg),
            HookError::InvalidToolOutput(msg) => write!(f, "Invalid tool output: {}", msg),
        }
    }
}

impl std::error::Error for HookError {}

#[derive(Debug, Clone)]
pub enum Hook {
    PostToolUse(PostToolUseContext),
    SessionStart(SessionStartContext),
}

#[derive(Debug, Clone)]
pub struct PostToolUseContext {
    pub tool_name: String,
    pub tool_output: CallToolResult,
    pub status: ToolExecutionStatus,
    pub timestamp: u64,
    pub session_id: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecutionStatus {
    Success,
    Error,
    Partial,
}

#[derive(Debug, Clone)]
pub struct SessionStartContext {
    pub session_id: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct HookContext {
    pub hook: Hook,
}

impl PostToolUseContext {
    pub fn new(tool_name: String, tool_output: CallToolResult) -> Self {
        let status = if tool_output.is_error.unwrap_or(false) {
            ToolExecutionStatus::Error
        } else {
            ToolExecutionStatus::Success
        };

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            tool_name,
            tool_output,
            status,
            timestamp,
            session_id: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl SessionStartContext {
    pub fn new(session_id: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            session_id,
            timestamp,
        }
    }
}
