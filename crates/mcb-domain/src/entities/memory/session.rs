use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::OriginContext;

/// Summary of an agent session.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    /// Stores the id value.
    pub id: String,
    /// Stores the project id value.
    pub project_id: String,
    /// Stores the session id value.
    pub session_id: String,
    /// Stores the topics value.
    pub topics: Vec<String>,
    /// Stores the decisions value.
    pub decisions: Vec<String>,
    /// Stores the next steps value.
    pub next_steps: Vec<String>,
    /// Stores the key files value.
    pub key_files: Vec<String>,
    /// Contextual information about the origin of the session.
    #[allow(missing_docs)]
    pub origin_context: Option<OriginContext>,
    /// Stores the created at value.
    pub created_at: i64,
}
