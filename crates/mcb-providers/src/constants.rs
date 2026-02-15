//! Provider Constants
//!
//! Constants specific to provider implementations. These are separated from
//! domain constants (which live in mcb-domain) and infrastructure constants.

// ============================================================================
// EMBEDDING API CONSTANTS
// ============================================================================

/// VoyageAI max input tokens
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// VoyageAI max output tokens
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// OpenAI max tokens per request
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Anthropic (Voyage AI) max input tokens per request
pub const ANTHROPIC_MAX_INPUT_TOKENS: usize = 32000;

/// Ollama server default port
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

// ============================================================================
// CACHE PROVIDER CONSTANTS
// ============================================================================

/// Redis default port
pub const REDIS_DEFAULT_PORT: u16 = 6379;

// ============================================================================
// EVENTS PROVIDER CONSTANTS
// ============================================================================

/// NATS default subject for domain events
pub const NATS_DEFAULT_SUBJECT: &str = "mcb.events";

/// Tokio broadcast event bus default channel capacity
pub const EVENTS_TOKIO_DEFAULT_CAPACITY: usize = 1024;

// ============================================================================
// LANGUAGE PROVIDER CONSTANTS
// ============================================================================

/// Default max chunk size (lines)
pub const LANGUAGE_DEFAULT_MAX_CHUNK_SIZE: usize = 50;

/// Maximum chunks per file
pub const LANGUAGE_MAX_CHUNKS_PER_FILE: usize = 75;

/// Priority threshold for chunk filtering
pub const LANGUAGE_PRIORITY_THRESHOLD: usize = 50;

// ============================================================================
// HTTP CONSTANTS
// ============================================================================

/// HTTP request timeout error message template
pub const ERROR_MSG_REQUEST_TIMEOUT: &str = "Request timed out after {:?}";

// ============================================================================
// EDGEVEC VECTOR STORE CONSTANTS
// ============================================================================

/// EdgeVec HNSW M parameter (max connections per node in layers > 0)
pub const EDGEVEC_HNSW_M: u32 = 16;

/// EdgeVec HNSW M0 parameter (max connections per node in layer 0)
pub const EDGEVEC_HNSW_M0: u32 = 32;

/// EdgeVec HNSW ef_construction parameter
pub const EDGEVEC_HNSW_EF_CONSTRUCTION: u32 = 200;

/// EdgeVec HNSW ef_search parameter
pub const EDGEVEC_HNSW_EF_SEARCH: u32 = 64;

/// EdgeVec default dimensions (for OpenAI embeddings)
pub const EDGEVEC_DEFAULT_DIMENSIONS: usize = 1536;

// ============================================================================
// MILVUS VECTOR STORE CONSTANTS
// ============================================================================

/// Milvus field varchar max length
pub const MILVUS_FIELD_VARCHAR_MAX_LENGTH: i32 = 512;

/// Milvus metadata varchar max length
pub const MILVUS_METADATA_VARCHAR_MAX_LENGTH: i32 = 65535;

/// Milvus IvfFlat nlist parameter
pub const MILVUS_IVFFLAT_NLIST: u32 = 128;

/// Milvus default port
pub const MILVUS_DEFAULT_PORT: u16 = 19530;

/// Milvus default connection timeout in seconds
pub const MILVUS_DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Milvus default query limit for aggregation queries
pub const MILVUS_DEFAULT_QUERY_LIMIT: i64 = 10_000;

/// Milvus pagination batch size (keeps gRPC responses under 4MB limit)
pub const MILVUS_QUERY_BATCH_SIZE: usize = 100;

// ============================================================================
// QDRANT VECTOR STORE CONSTANTS
// ============================================================================

/// Qdrant default server port
pub const QDRANT_DEFAULT_PORT: u16 = 6333;
