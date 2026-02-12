//! Provides observation domain definitions.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::memory::{ExecutionMetadata, OriginContext, QualityGateResult};

/// Categorizes the type of observation recorded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationType {
    /// Represents a code snippet or file content.
    Code,
    /// Represents a recorded decision.
    Decision,
    /// Represents general project context or information.
    Context,
    /// Represents an error or exception.
    Error,
    /// Represents a summary or aggregation of data.
    Summary,
    /// Represents an execution trace or log.
    Execution,
    /// Represents a quality gate check result.
    QualityGate,
}

impl ObservationType {
    /// Returns the string representation of the observation type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use mcb_domain::ObservationType;
    /// assert_eq!(ObservationType::Code.as_str(), "code");
    /// assert_eq!(ObservationType::Decision.as_str(), "decision");
    /// ```
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

    /// Parses a string into an `ObservationType`.
    ///
    /// # Errors
    ///
    /// Returns an error if the string does not match any known observation type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use mcb_domain::ObservationType;
    /// assert!(ObservationType::from_str("code").is_ok());
    /// assert!(ObservationType::from_str("invalid").is_err());
    /// ```
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

/// Metadata associated with an observation.
///
/// Contains contextual information about where and when an observation was recorded,
/// including session, repository, file, and git information.
#[derive(Clone, Serialize, Deserialize)]
pub struct ObservationMetadata {
    /// Unique identifier for the metadata.
    pub id: String,
    /// Identifier of the session where the observation occurred.
    pub session_id: Option<String>,
    /// Identifier of the repository related to the observation.
    pub repo_id: Option<String>,
    /// Path to the file related to the observation.
    pub file_path: Option<String>,
    /// Git branch name.
    pub branch: Option<String>,
    /// Git commit hash.
    pub commit: Option<String>,
    /// Details about the tool execution (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionMetadata>,
    /// Details about the quality gate result (if applicable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality_gate: Option<QualityGateResult>,
    /// Contextual information about the origin of the observation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin_context: Option<OriginContext>,
}

impl std::fmt::Debug for ObservationMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObservationMetadata")
            .field("id", &self.id)
            .field("session_id", &self.session_id.as_ref().map(|_| "REDACTED"))
            .field("repo_id", &self.repo_id)
            .field("file_path", &self.file_path)
            .field("branch", &self.branch)
            .field("commit", &self.commit)
            .field("execution", &self.execution)
            .field("quality_gate", &self.quality_gate)
            .field("origin_context", &self.origin_context)
            .finish()
    }
}

impl Default for ObservationMetadata {
    /// Creates a new `ObservationMetadata` with a generated UUID and all optional fields set to `None`.
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
            origin_context: None,
        }
    }
}

/// A recorded unit of knowledge or event in the system.
///
/// Observations are immutable records of code, decisions, context, errors, summaries,
/// execution traces, or quality gate results. Each observation is uniquely identified,
/// tagged for semantic search, and includes metadata about its origin and context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    /// Unique identifier for the observation.
    pub id: String,
    /// Identifier of the project this observation belongs to.
    pub project_id: String,
    /// The actual content or payload of the observation.
    pub content: String,
    /// Hash of the content for deduplication.
    pub content_hash: String,
    /// List of semantic tags.
    pub tags: Vec<String>,
    /// Classification of the observation.
    #[serde(rename = "type", alias = "observation_type")]
    pub r#type: ObservationType,
    /// Additional metadata.
    pub metadata: ObservationMetadata,
    /// Timestamp when the observation was created (Unix epoch).
    pub created_at: i64,
    /// Reference to vector embedding.
    pub embedding_id: Option<String>,
}
