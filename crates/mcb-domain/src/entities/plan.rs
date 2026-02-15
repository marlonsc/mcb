//! Plan Domain Entities
//!
//! This module defines the entities used for high-level planning and architectural
//! decision making. It supports versioning and review workflows to manage the
//! lifecycle of strategic initiatives.
//!
//! # Core Entities
//! - [`Plan`]: The high-level container for a strategic initiative.
//! - [`PlanVersion`]: An immutable snapshot of the plan content.
//! - [`PlanReview`]: A formal approval/rejection record for a specific version.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::EntityMetadata;

/// A plan definition owned by an organization and project.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Plan {
    /// Common entity metadata (id, timestamps).
    #[serde(flatten)]
    pub metadata: EntityMetadata,
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
}

impl_base_entity!(Plan);

/// Lifecycle status for a plan.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
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
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

/// A versioned snapshot of a plan's content.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlanVersion {
    /// Unique identifier (UUID).
    pub id: String,
    /// Organization that owns this plan version.
    pub org_id: String,
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
    /// Organization that owns this review.
    pub org_id: String,
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
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    strum_macros::Display,
    strum_macros::AsRefStr,
    strum_macros::EnumString,
)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case", ascii_case_insensitive)]
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
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}
