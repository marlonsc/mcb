//! Provides ids domain definitions.
use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Serialize,
            Deserialize,
            schemars::JsonSchema,
        )]
        pub struct $name(String);

        impl $name {
            /// Create a new instance
            pub fn new<S: Into<String>>(id: S) -> Self {
                Self(id.into())
            }

            /// Get the string representation
            pub fn as_str(&self) -> &str {
                &self.0
            }

            /// Convert into string
            pub fn into_string(self) -> String {
                self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<String> for $name {
            fn from(s: String) -> Self {
                Self::new(s)
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::new(s)
            }
        }

        impl AsRef<str> for $name {
            fn as_ref(&self) -> &str {
                self.as_str()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0
            }
        }
    };
}

define_id!(CollectionId, "Strong typed identifier for a collection");
define_id!(ChunkId, "Strong typed identifier for a code chunk");
define_id!(RepositoryId, "Strong typed identifier for a VCS repository");
define_id!(
    SessionId,
    "Strong typed identifier for an agent or workflow session"
);
define_id!(
    ObservationId,
    "Strong typed identifier for a memory observation"
);
define_id!(
    OperationId,
    "Strong typed identifier for an indexing operation"
);
define_id!(
    OrgId,
    "Strong typed identifier for an organization (tenant isolation)"
);
define_id!(UserId, "Strong typed identifier for a user");
define_id!(TeamId, "Strong typed identifier for a team");
define_id!(ApiKeyId, "Strong typed identifier for an API key");
define_id!(BranchId, "Strong typed identifier for a tracked branch");
define_id!(WorktreeId, "Strong typed identifier for a worktree");
define_id!(
    AssignmentId,
    "Strong typed identifier for an agent-worktree assignment"
);
define_id!(PlanId, "Strong typed identifier for a plan");
define_id!(PlanVersionId, "Strong typed identifier for a plan version");
define_id!(PlanReviewId, "Strong typed identifier for a plan review");
define_id!(IssueId, "Strong typed identifier for a project issue");
define_id!(
    IssueCommentId,
    "Strong typed identifier for an issue comment"
);
define_id!(IssueLabelId, "Strong typed identifier for an issue label");
