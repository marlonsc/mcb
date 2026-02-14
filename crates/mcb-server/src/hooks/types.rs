//! Type definitions for the hook system.
//!
//! Includes hook event types, contexts, and errors.

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use mcb_domain::value_objects::ids::SessionId;
use rmcp::model::CallToolResult;
use thiserror::Error;

/// Result type for hook operations.
pub type HookResult<T> = Result<T, HookError>;

/// Errors that can occur during hook execution.
#[derive(Debug, Clone, Error)]
pub enum HookError {
    /// The memory service is not available.
    #[error("Memory service unavailable")]
    MemoryServiceUnavailable,
    /// Failed to store an observation in memory.
    #[error("Failed to store observation: {0}")]
    FailedToStoreObservation(String),
    /// Failed to inject context into the session.
    #[error("Failed to inject context: {0}")]
    FailedToInjectContext(String),
    /// The tool output provided is invalid.
    #[error("Invalid tool output: {0}")]
    InvalidToolOutput(String),
}

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
    pub session_id: Option<SessionId>,
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
    pub session_id: SessionId,
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
    pub fn with_session_id<S: Into<SessionId>>(mut self, session_id: S) -> Self {
        self.session_id = Some(session_id.into());
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
    pub fn new(session_id: SessionId) -> Self {
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
