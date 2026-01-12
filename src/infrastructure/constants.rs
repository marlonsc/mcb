//! Application-wide constants and default values
//!
//! Centralizes configuration of timeout values, limits, and other magic numbers
//! to ensure consistency across the codebase and enable easy customization.

use std::time::Duration;

// ============================================================================
// HTTP Request Timeouts
// ============================================================================

/// Default HTTP request timeout for embedding providers (OpenAI, Ollama, etc.)
///
/// This timeout applies to API calls that should complete relatively quickly.
/// Embedding requests typically take 100-500ms depending on model size.
/// 30 seconds provides ample margin for network latency and model inference.
pub const HTTP_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// HTTP request timeout for indexing operations
///
/// Large codebase indexing can take significant time due to:
/// - File system I/O
/// - AST parsing
/// - Embedding generation
/// - Vector database writes
///
/// 5 minutes (300 seconds) is a reasonable upper bound for most codebases.
/// This can be overridden per-request if needed.
pub const INDEXING_OPERATION_TIMEOUT: Duration = Duration::from_secs(300);

// ============================================================================
// Cache TTL (Time-To-Live) Values
// ============================================================================

/// Cache TTL for search results
///
/// Search results are cached to avoid redundant embedding and vector searches.
/// 30 minutes balances freshness with cache hit rates. Long enough to cover
/// multiple queries in a typical analysis session, short enough that stale
/// results aren't served for long.
pub const SEARCH_RESULT_CACHE_TTL: Duration = Duration::from_secs(1800);

// ============================================================================
// Search Configuration
// ============================================================================

/// Minimum query length in characters
///
/// Prevents single-character or trivial queries from consuming resources.
pub const SEARCH_QUERY_MIN_LENGTH: usize = 3;

/// Maximum query length in characters
///
/// Prevents extremely long queries that could cause performance issues
/// in embedding generation or vector search.
pub const SEARCH_QUERY_MAX_LENGTH: usize = 1000;

/// Maximum number of search results to return
///
/// Even if user requests more, results are clamped to this value to ensure
/// reasonable response sizes and prevent excessive resource consumption.
pub const SEARCH_RESULT_LIMIT_MAX: usize = 50;

/// Minimum acceptable search result limit (must request at least 1)
pub const SEARCH_RESULT_LIMIT_MIN: usize = 1;

// ============================================================================
// FastEmbed Model Configuration
// ============================================================================

/// FastEmbed model maximum token length
///
/// FastEmbed models have a fixed maximum token length. This is model-specific.
/// Default FastEmbed models (like BAAI/bge-base-en-v1.5) support up to 512 tokens.
/// Note: This should ideally come from model introspection API, not hardcoded.
pub const FASTEMBED_MAX_TOKENS: usize = 512;

/// FastEmbed model embedding dimension
///
/// Default FastEmbed models produce embeddings of this dimension.
/// This is model-specific. BAAI/bge-base-en-v1.5 produces 768-dimensional embeddings.
/// Note: This should ideally come from model introspection API, not hardcoded.
pub const FASTEMBED_EMBEDDING_DIMENSION: usize = 384;

// ============================================================================
// Protected Collection Names
// ============================================================================

/// System-reserved collection name (cannot be cleared)
pub const PROTECTED_COLLECTION_SYSTEM: &str = "system";

/// Admin-reserved collection name (cannot be cleared)
pub const PROTECTED_COLLECTION_ADMIN: &str = "admin";

// ============================================================================
// Provider Default URLs
// ============================================================================

/// Default Ollama API base URL
///
/// Ollama runs locally on port 11434 by default. This default is used
/// when no explicit URL is provided in configuration.
pub const OLLAMA_DEFAULT_URL: &str = "http://localhost:11434";
