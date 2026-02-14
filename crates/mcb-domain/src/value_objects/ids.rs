//! Strong-typed UUID identifiers for all domain entities.
use derive_more::{Display, From, Into};
use serde::{Deserialize, Serialize};

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            Hash,
            Display,
            From,
            Into,
            Serialize,
            Deserialize,
            schemars::JsonSchema,
        )]
        #[display("{_0}")]
        pub struct $name(uuid::Uuid);

        impl $name {
            /// Generate a new random UUID v4 identifier.
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }

            /// Wrap an existing [`uuid::Uuid`].
            pub fn from_uuid(uuid: uuid::Uuid) -> Self {
                Self(uuid)
            }

            /// Derive a deterministic v5 UUID from a human-readable name.
            ///
            /// The namespace is scoped per type so `CollectionId::from_name("x")`
            /// and `SessionId::from_name("x")` produce different UUIDs.
            pub fn from_name(name: &str) -> Self {
                let ns =
                    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, stringify!($name).as_bytes());
                Self(uuid::Uuid::new_v5(&ns, name.as_bytes()))
            }

            /// Parse from any string: tries UUID first, falls back to `from_name`.
            pub fn from_string(s: &str) -> Self {
                match uuid::Uuid::parse_str(s) {
                    Ok(u) => Self(u),
                    Err(_) => Self::from_name(s),
                }
            }

            /// Hyphenated UUID string (allocates).
            pub fn as_str(&self) -> String {
                self.0.to_string()
            }

            /// Consume and return the hyphenated string.
            pub fn into_string(self) -> String {
                self.0.to_string()
            }

            /// Access the inner [`uuid::Uuid`].
            pub fn inner(&self) -> uuid::Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<$name> for String {
            fn from(id: $name) -> Self {
                id.0.to_string()
            }
        }

        impl std::str::FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                uuid::Uuid::parse_str(s).map(Self)
            }
        }

        impl AsRef<uuid::Uuid> for $name {
            fn as_ref(&self) -> &uuid::Uuid {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(s: &str) -> Self {
                Self::from_string(s)
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

define_id!(CodebaseId, "Strong typed identifier for a codebase");
define_id!(FileId, "Strong typed identifier for a file");
define_id!(FunctionId, "Strong typed identifier for a function");
define_id!(ClassId, "Strong typed identifier for a class");

define_id!(TeamMemberId, "Strong typed identifier for a team member");
define_id!(
    IssueLabelAssignmentId,
    "Strong typed identifier for an issue label assignment"
);
