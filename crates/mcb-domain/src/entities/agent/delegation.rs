use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Represents a delegation of work from a parent agent session to a child session.
///
/// A delegation captures the complete lifecycle of delegating a task to another agent,
/// including the prompt sent, the result received, and timing information.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Delegation {
    /// Unique identifier for this delegation.
    pub id: String,
    /// ID of the parent session that initiated the delegation.
    pub parent_session_id: String,
    /// ID of the child session that executed the delegated work.
    pub child_session_id: String,
    /// The prompt/instructions sent to the child session.
    pub prompt: String,
    /// Optional ID of the embedding for the prompt (for semantic search/retrieval).
    pub prompt_embedding_id: Option<String>,
    /// The result returned from the child session, if available.
    pub result: Option<String>,
    /// Whether the delegation completed successfully.
    pub success: bool,
    /// Unix timestamp (milliseconds) when the delegation was created.
    pub created_at: i64,
    /// Unix timestamp (milliseconds) when the delegation completed, if applicable.
    pub completed_at: Option<i64>,
    /// Duration of the delegation execution in milliseconds, if completed.
    pub duration_ms: Option<i64>,
}
