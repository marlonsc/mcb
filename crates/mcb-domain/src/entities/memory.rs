//! Memory entities for observation storage and session tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Observation type for semantic memory (code, decision, context, error, summary, execution, quality_gate).
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

/// Execution type for command tracking (test, lint, build, CI).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionType {
    Test,
    Lint,
    Build,
    CI,
}

/// Status for quality gates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityGateStatus {
    Passed,
    Failed,
    Warning,
    Skipped,
}

impl QualityGateStatus {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Warning => "warning",
            Self::Skipped => "skipped",
        }
    }
}

impl std::str::FromStr for QualityGateStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "warning" => Ok(Self::Warning),
            "skipped" => Ok(Self::Skipped),
            _ => Err(format!("Unknown quality gate status: {s}")),
        }
    }
}

impl ExecutionType {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Test => "test",
            Self::Lint => "lint",
            Self::Build => "build",
            Self::CI => "ci",
        }
    }
}

impl std::str::FromStr for ExecutionType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "test" => Ok(Self::Test),
            "lint" => Ok(Self::Lint),
            "build" => Ok(Self::Build),
            "ci" => Ok(Self::CI),
            _ => Err(format!("Unknown execution type: {s}")),
        }
    }
}

/// Metadata for execution tracking stored on execution observations.
/// Has identity per Clean Architecture (CA004) for traceability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Entity identity for CA004 compliance and traceability.
    #[serde(default)]
    pub id: String,
    pub command: String,
    pub exit_code: Option<i32>,
    pub duration_ms: Option<i64>,
    pub success: bool,
    pub execution_type: ExecutionType,
    pub coverage: Option<f32>,
    pub files_affected: Vec<String>,
    pub output_summary: Option<String>,
    pub warnings_count: Option<i32>,
    pub errors_count: Option<i32>,
}

/// Quality gate result metadata stored on quality gate observations.
/// Has identity per Clean Architecture (CA004) for traceability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGateResult {
    /// Entity identity for CA004 compliance and traceability.
    #[serde(default)]
    pub id: String,
    pub gate_name: String,
    pub status: QualityGateStatus,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
    pub execution_id: Option<String>,
}

/// Metadata for an observation (session, repo, file, branch, commit).
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

/// Memory observation stored in SQLite with optional embedding reference for RAG search.
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

/// Summary of a conversation session with topics, decisions, and action items.
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

/// Memory search hit: observation plus similarity score.
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

/// Filter specification for memory queries (value object; optional id for tracing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryFilter {
    /// Optional client-provided id for idempotency/tracing (satisfies CA004 entity identity).
    pub id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub observation_type: Option<ObservationType>,
    pub session_id: Option<String>,
    pub repo_id: Option<String>,
    pub time_range: Option<(i64, i64)>,
    pub branch: Option<String>,
    pub commit: Option<String>,
}

// ============================================================================
// Phase 4: Error Pattern Memory (ADR-032)
// ============================================================================

/// Category of error pattern for classification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorPatternCategory {
    /// Compiler/type errors
    Compilation,
    /// Runtime exceptions
    Runtime,
    /// Test failures
    Test,
    /// Lint/style violations
    Lint,
    /// Build system errors
    Build,
    /// Configuration errors
    Config,
    /// Network/API errors
    Network,
    /// Other uncategorized errors
    Other,
}

impl ErrorPatternCategory {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compilation => "compilation",
            Self::Runtime => "runtime",
            Self::Test => "test",
            Self::Lint => "lint",
            Self::Build => "build",
            Self::Config => "config",
            Self::Network => "network",
            Self::Other => "other",
        }
    }
}

impl std::str::FromStr for ErrorPatternCategory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compilation" => Ok(Self::Compilation),
            "runtime" => Ok(Self::Runtime),
            "test" => Ok(Self::Test),
            "lint" => Ok(Self::Lint),
            "build" => Ok(Self::Build),
            "config" => Ok(Self::Config),
            "network" => Ok(Self::Network),
            "other" => Ok(Self::Other),
            _ => Err(format!("Unknown error pattern category: {s}")),
        }
    }
}

/// Recurring error pattern with solutions learned from past resolutions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub id: String,
    pub project_id: String,
    /// Normalized error message for pattern matching.
    pub pattern_signature: String,
    pub description: String,
    pub category: ErrorPatternCategory,
    pub solutions: Vec<String>,
    pub affected_files: Vec<String>,
    pub tags: Vec<String>,
    pub occurrence_count: i64,
    pub first_seen_at: i64,
    pub last_seen_at: i64,
    pub embedding_id: Option<String>,
}

/// Link between a specific error observation and a recognized pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPatternMatch {
    pub id: String,
    pub pattern_id: String,
    pub observation_id: String,
    /// Confidence score scaled by 1000 (e.g., 950 = 0.95).
    pub confidence: i64,
    pub solution_applied: Option<i32>,
    pub resolution_successful: Option<bool>,
    pub matched_at: i64,
    pub resolved_at: Option<i64>,
}
