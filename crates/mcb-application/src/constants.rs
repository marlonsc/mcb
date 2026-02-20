//!
//! **Documentation**: [docs/modules/application.md](../../../docs/modules/application.md)
//!
//! Application layer constants
//!
//! Constants used by use cases and domain services.

// ============================================================================
// MEMORY / SEARCH
// ============================================================================

/// Max length for observation content preview in search results
pub const OBSERVATION_PREVIEW_LENGTH: usize = 120;

/// Name of the vector collection for storing observations
pub const MEMORY_COLLECTION_NAME: &str = "memories";

// ============================================================================
// INDEXING
// ============================================================================

/// Directories to skip during codebase indexing
pub const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__"];

/// Publish progress event every N files
pub const PROGRESS_UPDATE_INTERVAL: usize = 10;
