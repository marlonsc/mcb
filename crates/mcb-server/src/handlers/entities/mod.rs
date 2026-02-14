//! Entity handlers module.

pub mod common;
pub mod issue;
pub mod org;
pub mod plan;
pub mod vcs;

pub use common::EntityHandler;
pub use issue::IssueEntityHandler;
pub use org::OrgEntityHandler;
pub use plan::PlanEntityHandler;
pub use vcs::VcsEntityHandler;
