//! External Provider Ports
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

/// Code analysis provider ports.
mod analysis;
/// Provider configuration manager ports.
mod config_manager;
/// Cryptographic provider ports.
mod crypto;
/// Embedding provider ports.
mod embedding;
/// HTTP client provider ports.
mod http;
/// Hybrid search provider ports.
mod hybrid_search;
/// Language-specific chunking provider ports.
mod language_chunking;
/// Metrics provider ports.
mod metrics;
/// Project detection provider ports.
mod project_detection;
/// Version control system provider ports.
mod vcs;
/// Vector store provider ports.
mod vector_store;

pub use analysis::*;
pub use config_manager::*;
pub use crypto::*;
pub use embedding::*;
pub use http::*;
pub use hybrid_search::*;
pub use language_chunking::*;
pub use metrics::*;
pub use project_detection::*;
pub use vcs::*;
pub use vector_store::*;
