//! Memory entities for observation storage and session tracking.

use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservationMetadata {
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub file_path: Option<String>,
    pub branch: Option<String>,
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
    pub observation: Observation,
    pub similarity_score: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    pub tags: Option<Vec<String>>,
    pub observation_type: Option<ObservationType>,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
}
