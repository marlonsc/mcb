//! Unified entity CRUD argument types.
//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)

pub mod issue;
pub mod org;
pub mod plan;
pub mod types;
pub mod vcs;

pub use issue::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
pub use org::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
pub use plan::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
pub use types::{EntityAction, EntityArgs, EntityResource};
pub use vcs::{VcsEntityAction, VcsEntityArgs, VcsEntityResource};
