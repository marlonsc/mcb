//! Plan, plan version, and plan review entities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A plan definition owned by an organization and project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Plan {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization that owns this plan.
    pub org_id: String,
    /// Project this plan belongs to.
    pub project_id: String,
    /// Human-readable title for the plan.
    pub title: String,
    /// Detailed plan description.
    pub description: String,
    /// Current lifecycle status.
    pub status: PlanStatus,
    /// User that created the plan.
    pub created_by: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
    /// Last update timestamp (Unix epoch).
    pub updated_at: i64,
}

/// Lifecycle status for a plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum PlanStatus {
    /// Plan is in draft state.
    Draft,
    /// Plan is active and ready to execute.
    Active,
    /// Plan is currently being executed.
    Executing,
    /// Plan execution is complete.
    Completed,
    /// Plan is archived.
    Archived,
}

impl PlanStatus {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Active => "active",
            Self::Executing => "executing",
            Self::Completed => "completed",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for PlanStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "draft" => Ok(Self::Draft),
            "active" => Ok(Self::Active),
            "executing" => Ok(Self::Executing),
            "completed" => Ok(Self::Completed),
            "archived" => Ok(Self::Archived),
            _ => Err(format!("Unknown plan status: {s}")),
        }
    }
}

/// A versioned snapshot of a plan's content.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanVersion {
    /// Unique identifier (UUID).
    pub id: String,
    /// Parent plan identifier.
    pub plan_id: String,
    /// Monotonic version number for the plan.
    pub version_number: i64,
    /// Serialized JSON payload for the version.
    pub content_json: String,
    /// Human summary of changes in this version.
    pub change_summary: String,
    /// User that created this version.
    pub created_by: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
}

/// A review decision for a specific plan version.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanReview {
    /// Unique identifier (UUID).
    pub id: String,
    /// Plan version that was reviewed.
    pub plan_version_id: String,
    /// Reviewer user identifier.
    pub reviewer_id: String,
    /// Review verdict string.
    pub verdict: ReviewVerdict,
    /// Reviewer feedback text.
    pub feedback: String,
    /// Creation timestamp (Unix epoch).
    pub created_at: i64,
}

/// Verdict values for a plan review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ReviewVerdict {
    /// Review approved the plan version.
    Approved,
    /// Review rejected the plan version.
    Rejected,
    /// Review requires revision before approval.
    NeedsRevision,
}

impl ReviewVerdict {
    /// Returns the string representation.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::NeedsRevision => "needs_revision",
        }
    }
}

impl std::fmt::Display for ReviewVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ReviewVerdict {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "approved" => Ok(Self::Approved),
            "rejected" => Ok(Self::Rejected),
            "needs_revision" => Ok(Self::NeedsRevision),
            _ => Err(format!("Unknown review verdict: {s}")),
        }
    }
}
