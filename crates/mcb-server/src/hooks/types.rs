use rmcp::model::CallToolResult;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Result type for hook operations.
pub type HookResult<T> = Result<T, HookError>;

/// Errors that can occur during hook execution.
#[derive(Debug, Clone)]
pub enum HookError {
    /// The memory service is not available.
    MemoryServiceUnavailable,
    /// Failed to store an observation in memory.
    FailedToStoreObservation(String),
    /// Failed to inject context into the session.
    FailedToInjectContext(String),
    /// The tool output provided is invalid.
    InvalidToolOutput(String),
}

impl std::fmt::Display for HookError {
    /// Formats the error message.
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

/// Represents the type of hook event being triggered.
#[derive(Debug, Clone)]
pub enum Hook {
    /// Triggered after a tool has been executed.
    PostToolUse(PostToolUseContext),
    /// Triggered when a new session starts.
    SessionStart(SessionStartContext),
}

/// Context data for the `PostToolUse` hook.
#[derive(Debug, Clone)]
pub struct PostToolUseContext {
    /// Name of the tool that was executed.
    pub tool_name: String,
    /// The result output from the tool execution.
    pub tool_output: CallToolResult,
    /// The execution status (success, error, partial).
    pub status: ToolExecutionStatus,
    /// Timestamp when the execution finished (Unix epoch).
    pub timestamp: u64,
    /// Optional ID of the session where the tool was used.
    pub session_id: Option<String>,
    /// Additional metadata associated with the execution.
    pub metadata: HashMap<String, String>,
}

/// Status of a tool execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolExecutionStatus {
    /// Tool completed successfully.
    Success,
    /// Tool execution failed.
    Error,
    /// Tool completed with partial results or warnings.
    Partial,
}

/// Context data for the `SessionStart` hook.
#[derive(Debug, Clone)]
pub struct SessionStartContext {
    /// Unique identifier for the new session.
    pub session_id: String,
    /// Timestamp when the session started (Unix epoch).
    pub timestamp: u64,
}

/// Container for the specific hook event context.
#[derive(Debug, Clone)]
pub struct HookContext {
    /// The specific hook event.
    pub hook: Hook,
}

impl PostToolUseContext {
    /// Creates a new `PostToolUseContext` from a tool name and output.
    ///
    /// Automatically determines the status based on `tool_output.is_error`.
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

    /// Sets the session ID for the context.
    #[must_use]
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Adds a key-value pair to the metadata.
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }
}

impl SessionStartContext {
    /// Creates a new `SessionStartContext` for a given session ID.
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
