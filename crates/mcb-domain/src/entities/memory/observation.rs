use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::execution::ExecutionMetadata;
use super::quality_gate::QualityGateResult;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationType {
    Code,
    Decision,
    Context,
    Error,
    Summary,
    Execution,
    QualityGate,
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
            Self::Execution => "execution",
            Self::QualityGate => "quality_gate",
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
            "execution" => Ok(Self::Execution),
            "quality_gate" => Ok(Self::QualityGate),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionMetadata>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_gate: Option<QualityGateResult>,
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
            execution: None,
            quality_gate: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: String,
    pub project_id: String,
    pub content: String,
    pub content_hash: String,
    pub tags: Vec<String>,
    pub observation_type: ObservationType,
    pub metadata: ObservationMetadata,
    pub created_at: i64,
    pub embedding_id: Option<String>,
}
