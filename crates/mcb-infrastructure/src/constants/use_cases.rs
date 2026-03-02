//! Use case constants for service implementations.
//!
//! Constants used by the application-layer use case services that live
//! in service registry builders.

// ============================================================================
// MEMORY / SEARCH
// ============================================================================

/// Max length for observation content preview in search results.
pub const OBSERVATION_PREVIEW_LENGTH: usize = 120;

/// Name of the vector collection for storing observations.
pub const MEMORY_COLLECTION_NAME: &str = "memories";

// ============================================================================
// INDEXING
// ============================================================================

/// Directories to skip during codebase indexing.
pub const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "__pycache__"];
