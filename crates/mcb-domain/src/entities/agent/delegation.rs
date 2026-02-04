use serde::{Deserialize, Serialize};

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
