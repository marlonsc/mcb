//! Administrative interfaces for system management and monitoring.
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

/// Dashboard/analytics query ports.
mod dashboard;
/// Indexing operation tracking ports.
mod indexing;
/// Provider admin interfaces (embedding, vector store, language).
mod provider_admin;
/// Validation operation tracking ports.
mod validation;

pub use dashboard::*;
pub use indexing::*;
pub use provider_admin::*;
pub use validation::*;
