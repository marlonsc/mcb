//! Type definitions for the hook system.
//!
//! Includes hook event types, contexts, and errors.

use std::collections::HashMap;

use mcb_domain::utils::time as domain_time;
use mcb_domain::value_objects::ids::SessionId;
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
    /// Creates a new `PostToolUseContext` from a tool name and error status.
    ///
    /// This avoids cloning the entire `CallToolResult` — only the `is_error`
    /// flag is needed by the hook processor.
    pub fn new(tool_name: String, is_error: bool) -> Self {
        let status = if is_error {
            ToolExecutionStatus::Error
        } else {
            ToolExecutionStatus::Success
        };

        let timestamp =
            domain_time::epoch_secs_u64().unwrap_or_else(|e| panic!("system clock failure: {e}"));

        Self {
            tool_name,
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
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl SessionStartContext {
    /// Creates a new `SessionStartContext` for a given session ID.
    pub fn new(session_id: SessionId) -> Self {
        let timestamp =
            domain_time::epoch_secs_u64().unwrap_or_else(|e| panic!("system clock failure: {e}"));

        Self {
            session_id,
            timestamp,
        }
    }
}
