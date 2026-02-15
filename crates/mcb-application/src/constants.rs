//! Application layer constants
//!
//! Constants used by use cases and domain services.

// ============================================================================
// MEMORY / SEARCH
// ============================================================================

/// Reciprocal Rank Fusion (RRF) k parameter for hybrid result fusion
pub const RRF_K: f32 = 60.0;

/// Multiplier for hybrid search candidate retrieval (semantic * N)
pub const HYBRID_SEARCH_MULTIPLIER: usize = 3;

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
