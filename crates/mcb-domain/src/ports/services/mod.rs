//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md#service-ports)
//!
//! Domain service port interfaces for core business operations.

/// Agent session lifecycle management.
mod agent;
/// Browse and highlight operations.
mod browse;
/// Code chunking operations.
mod chunking;
/// Code intelligence / context operations.
mod context;
/// File hash state management.
mod hash;
/// Codebase indexing operations.
mod indexing;
/// Background job lifecycle management.
mod job;
/// Memory / observation storage and search.
mod memory;
/// Project detection operations.
mod project;
/// Semantic code search operations.
mod search;
/// Architecture validation operations.
mod validation_service;

pub use agent::*;
pub use browse::*;
pub use chunking::*;
pub use context::*;
pub use hash::*;
pub use indexing::*;
pub use job::*;
pub use memory::*;
pub use project::*;
pub use search::*;
pub use validation_service::*;
