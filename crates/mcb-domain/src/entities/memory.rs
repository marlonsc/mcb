//! Memory entities for observation storage and session tracking.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationType {
    Code,
    Decision,
    Context,
    Error,
    Summary,
}

impl ObservationType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Decision => "decision",
            Self::Context => "context",
            Self::Error => "error",
            Self::Summary => "summary",
        }
    }
}

impl std::str::FromStr for ObservationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "code" => Ok(Self::Code),
            "decision" => Ok(Self::Decision),
            "context" => Ok(Self::Context),
            "error" => Ok(Self::Error),
            "summary" => Ok(Self::Summary),
            _ => Err(format!("Unknown observation type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservationMetadata {
    pub id: String,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub file_path: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
}

impl Default for ObservationMetadata {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            session_id: None,
            repo_id: None,
            file_path: None,
            branch: None,
            commit: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub content: String,
    pub content_hash: String,
    pub tags: Vec<String>,
    pub observation_type: ObservationType,
    pub metadata: ObservationMetadata,
    pub created_at: i64,
    pub embedding_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub session_id: String,
    pub topics: Vec<String>,
    pub decisions: Vec<String>,
    pub next_steps: Vec<String>,
    pub key_files: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchResult {
    pub id: String,
    pub observation: Observation,
    pub similarity_score: f32,
}

/// Token-efficient memory search index result
///
/// This is a lightweight version of MemorySearchResult designed for
/// the 3-layer workflow (search -> timeline -> details). It returns
/// only essential metadata to minimize token usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySearchIndex {
    /// Observation ID (use with memory_get_observations for full details)
    pub id: String,
    /// Observation type (code, decision, context, error, summary)
    pub observation_type: String,
    /// Relevance score from hybrid search (0.0 to 1.0)
    pub relevance_score: f32,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Brief content preview (first N chars, truncated with ellipsis)
    pub content_preview: String,
    /// Associated session ID
    pub session_id: Option<String>,
    /// Associated repository ID
    pub repo_id: Option<String>,
    /// File path if applicable
    pub file_path: Option<String>,
    /// Creation timestamp
    pub created_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub tags: Option<Vec<String>>,
    pub observation_type: Option<ObservationType>,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub branch: Option<String>,
    pub commit: Option<String>,
}
