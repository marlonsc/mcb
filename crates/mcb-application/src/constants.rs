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

/// Indexing status: started
pub const INDEXING_STATUS_STARTED: &str = "started";

/// Indexing status: completed
pub const INDEXING_STATUS_COMPLETED: &str = "completed";

// ============================================================================
// RRF CALCULATION
// ============================================================================

/// RRF score numerator (1.0 / (rank + k)).
pub const RRF_SCORE_NUMERATOR: f32 = 1.0;

/// Maximum possible RRF score multiplier (2 search streams).
pub const RRF_MAX_SCORE_STREAMS: f32 = 2.0;

/// Normalized RRF score ceiling.
pub const RRF_NORMALIZED_MAX: f32 = 1.0;

/// Over-fetch multiplier for search filtering.
pub const SEARCH_OVERFETCH_MULTIPLIER: usize = 2;
