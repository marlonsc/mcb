use serde::{Deserialize, Serialize};

use super::observation::{Observation, ObservationType};

/// Result of a memory search query containing a matched observation and its similarity score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    /// Unique identifier for the search result.
    pub id: String,
    /// The observation entity that matched the search query.
    pub observation: Observation,
    /// Similarity score between the query and this observation (0.0 to 1.0).
    pub similarity_score: f32,
}

/// Index entry for a memory observation used in search operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchIndex {
    /// Unique identifier for the index entry.
    pub id: String,
    /// Type of observation (e.g., "code_snippet", "error", "decision").
    pub observation_type: String,
    /// Relevance score for ranking search results.
    pub relevance_score: f32,
    /// Tags associated with the observation for filtering and categorization.
    pub tags: Vec<String>,
    /// Preview of the observation content for display purposes.
    pub content_preview: String,
    /// Optional session identifier if the observation is tied to a specific session.
    pub session_id: Option<String>,
    /// Optional repository identifier if the observation is tied to a specific repository.
    pub repo_id: Option<String>,
    /// Optional file path if the observation is tied to a specific file.
    pub file_path: Option<String>,
    /// Timestamp (in milliseconds) when the observation was created.
    pub created_at: i64,
}

/// Filter criteria for querying memory observations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    /// Filter by specific observation identifier.
    pub id: Option<String>,
    /// Filter by one or more tags.
    pub tags: Option<Vec<String>>,
    /// Filter by observation type.
    pub observation_type: Option<ObservationType>,
    /// Filter by session identifier.
    pub session_id: Option<String>,
    /// Filter by repository identifier.
    pub repo_id: Option<String>,
    /// Filter by time range (start_ms, end_ms).
    pub time_range: Option<(i64, i64)>,
    /// Filter by git branch name.
    pub branch: Option<String>,
    /// Filter by git commit hash.
    pub commit: Option<String>,
}
