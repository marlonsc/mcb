//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Domain layer constants definitions

// ============================================================================
// INDEXING DOMAIN CONSTANTS
// ============================================================================

/// Default batch size for indexing operations
pub const INDEXING_BATCH_SIZE: usize = 10;

/// Indexing status: started
pub const INDEXING_STATUS_STARTED: &str = "started";

/// Indexing status: completed
pub const INDEXING_STATUS_COMPLETED: &str = "completed";

/// Minimum character length for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LENGTH: usize = 25;

/// Minimum number of lines for a code chunk to be indexed
pub const INDEXING_CHUNK_MIN_LINES: usize = 2;

/// Maximum number of chunks extracted from a single file
pub const INDEXING_CHUNKS_MAX_PER_FILE: usize = 50;

/// Maximum number of chunks returned for a single file when browsing vectors
pub const BROWSE_MAX_CHUNKS_PER_FILE: usize = 1000;

/// Default tombstone TTL for file hash cleanup (7 days in seconds).
pub const TOMBSTONE_TTL_SECS: u64 = 7 * 24 * 3600;

/// Default dashboard graph query limit.
pub const DEFAULT_DASHBOARD_LIMIT: usize = 30;

/// Default limit for session context search results.
pub const SESSION_SEARCH_LIMIT: usize = 10;

/// Milliseconds per second conversion factor.
pub const MS_PER_SEC: i64 = 1000;

// ============================================================================
// COMMON DEFAULTS
// ============================================================================

/// Default language identifier when language cannot be determined
pub const DEFAULT_LANGUAGE: &str = "unknown";

// ============================================================================
// PROVIDER DEFAULTS
// ============================================================================

/// Registry provider name for `SeaORM` database repositories.
pub const DEFAULT_DATABASE_PROVIDER: &str = "seaorm";

/// Registry provider name for universal language chunking.
pub const DEFAULT_LANGUAGE_PROVIDER: &str = "universal";

/// Registry provider name for Git VCS.
pub const DEFAULT_VCS_PROVIDER: &str = "git";

/// Registry provider name for hybrid search.
pub const DEFAULT_HYBRID_SEARCH_PROVIDER: &str = "default";

/// Registry provider name for indexing operations.
pub const DEFAULT_INDEXING_OP_PROVIDER: &str = "default";

/// Registry provider name for validation operations.
pub const DEFAULT_VALIDATION_OP_PROVIDER: &str = "default";

/// Default namespace for database repositories.
pub const DEFAULT_NAMESPACE: &str = "default";

/// Default limit for session queries to prevent unbounded result sets.
pub const DEFAULT_SESSION_LIMIT: u64 = 100;

/// Default limit for UI vector fetching.
pub const DEFAULT_BROWSE_LIMIT: usize = 50;

// ============================================================================
// PROVIDER SLUGS
// ============================================================================

/// Embedding provider slug: `OpenAI`.
pub const PROVIDER_SLUG_OPENAI: &str = "openai";

/// Embedding provider slug: Voyage AI.
pub const PROVIDER_SLUG_VOYAGEAI: &str = "voyageai";

/// Embedding provider slug: Anthropic.
pub const PROVIDER_SLUG_ANTHROPIC: &str = "anthropic";

/// Embedding provider slug: `FastEmbed` (local).
pub const PROVIDER_SLUG_FASTEMBED: &str = "fastembed";

/// Vector-store provider slug: `EdgeVec` (local).
pub const PROVIDER_SLUG_EDGEVEC: &str = "edgevec";

/// Vector-store provider slug: Milvus.
pub const PROVIDER_SLUG_MILVUS: &str = "milvus";

/// Vector-store provider slug: Qdrant.
pub const PROVIDER_SLUG_QDRANT: &str = "qdrant";

// ============================================================================
// CONFIG PROVIDER
// ============================================================================

/// Default configuration provider name (YAML file-based).
pub const DEFAULT_CONFIG_PROVIDER: &str = "loco_yaml";

// ============================================================================
// SERVICE NAMES (CA/DI registry)
// ============================================================================

/// Registry name for the context service.
pub const SERVICE_NAME_CONTEXT: &str = "context";

/// Registry name for the indexing service.
pub const SERVICE_NAME_INDEXING: &str = "indexing";

/// Registry name for the search service.
pub const SERVICE_NAME_SEARCH: &str = "search";

/// Registry name for the memory service.
pub const SERVICE_NAME_MEMORY: &str = "memory";

/// Registry name for the agent session service.
pub const SERVICE_NAME_AGENT_SESSION: &str = "agent_session";

/// Registry name for the validation service.
pub const SERVICE_NAME_VALIDATION: &str = "validation";

/// Registry name for the highlight service.
pub const SERVICE_NAME_HIGHLIGHT: &str = "highlight";

// ============================================================================
// DATABASE
// ============================================================================

/// `SQLite` in-memory DSN for test and fallback connections.
pub const SQLITE_MEMORY_DSN: &str = "sqlite::memory:";

// ============================================================================
// INDEXING OPERATION STATUS (database string enums)
// ============================================================================

/// Indexing-operation status: starting.
pub const INDEX_OP_STATUS_STARTING: &str = "starting";

/// Indexing-operation status: in progress.
pub const INDEX_OP_STATUS_IN_PROGRESS: &str = "in_progress";

/// Indexing-operation status: completed.
pub const INDEX_OP_STATUS_COMPLETED: &str = "completed";

/// Indexing-operation status: failed.
pub const INDEX_OP_STATUS_FAILED: &str = "failed";
