//! Plan Domain Entities
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#core-entities)
//!

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

crate::define_entity_org_project_audited! {
    /// A plan definition owned by an organization and project.
    #[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
    pub struct Plan {
        /// Title of the strategic plan.
        pub title: String,
        /// Detailed description of the plan.
        pub description: String,
        /// Current lifecycle status of the plan.
        pub status: PlanStatus,
        /// User identifier of the plan creator.
        pub created_by: String,
    }
}

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

crate::impl_as_str_from_as_ref!(PlanStatus);

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

crate::impl_as_str_from_as_ref!(ReviewVerdict);
