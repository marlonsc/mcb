use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub project_id: String,
    pub session_id: String,
    pub topics: Vec<String>,
    pub decisions: Vec<String>,
    pub next_steps: Vec<String>,
    pub key_files: Vec<String>,
    pub created_at: i64,
}
