use serde::{Deserialize, Serialize};

use super::observation::{Observation, ObservationType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub observation: Observation,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchIndex {
    pub id: String,
    pub observation_type: String,
    pub relevance_score: f32,
    pub tags: Vec<String>,
    pub content_preview: String,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub file_path: Option<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub observation_type: Option<ObservationType>,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub branch: Option<String>,
    pub commit: Option<String>,
}
