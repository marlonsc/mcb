//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Argument types and validators for MCP tool requests.

pub(crate) mod validation;
pub(crate) use validation::{validate_collection_name, validate_file_path};

/// Schema helper types for argument definitions.
pub mod schema_helpers;

/// Agent activity logging argument types.
pub mod agent;
/// Unified entity CRUD argument types.
pub mod entity;
/// Index operations argument types.
pub mod index;
/// Memory storage and retrieval argument types.
pub mod memory;
/// Project workflow argument types.
pub mod project;
/// Search operations argument types.
pub mod search;
/// Session lifecycle argument types.
pub mod session;
/// Validation and analysis argument types.
pub mod validate;
/// Version control operations argument types.
pub mod vcs;

// Re-export all types directly (no consolidated.rs indirection)
pub use agent::{AgentAction, AgentArgs, LogDelegationArgs, LogToolCallArgs};
pub use entity::{
    EntityAction, EntityArgs, EntityResource, IssueEntityAction, IssueEntityArgs,
    IssueEntityResource, OrgEntityAction, OrgEntityArgs, OrgEntityResource, PlanEntityAction,
    PlanEntityArgs, PlanEntityResource, VcsEntityAction, VcsEntityArgs, VcsEntityResource,
};
pub use index::{ClearIndexArgs, IndexAction, IndexArgs, IndexRepoArgs, IndexStatusArgs};
pub use memory::{
    GetMemoriesArgs, InjectContextArgs, ListMemoriesArgs, MemoryAction, MemoryArgs, MemoryResource,
    MemoryTimelineArgs, StoreMemoryArgs,
};
pub use project::{ProjectAction, ProjectArgs, ProjectResource};
pub use search::{SearchArgs, SearchCodeArgs, SearchMemoryArgs, SearchResource};
pub use session::{
    GetSessionArgs, ListSessionsArgs, SessionAction, SessionArgs, StartSessionArgs,
    SummarizeSessionArgs,
};
pub use validate::{
    AnalyzeCodeArgs, ListRulesArgs, ValidateAction, ValidateArgs, ValidateCodeArgs, ValidateScope,
};
pub use vcs::{AnalyzeImpactArgs, CompareBranchesArgs, ListReposArgs, VcsAction, VcsArgs};
