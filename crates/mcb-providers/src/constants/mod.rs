//! Provider Constants
//!
//! Constants specific to provider implementations. These are separated from
//! domain constants (which live in mcb-domain) and infrastructure constants.

// ============================================================================
// EMBEDDING API CONSTANTS
// ============================================================================

/// `VoyageAI` max input tokens
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// `VoyageAI` max output tokens
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// `OpenAI` max tokens per request
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Anthropic (Voyage AI) max input tokens per request
pub const ANTHROPIC_MAX_INPUT_TOKENS: usize = 32000;

/// Ollama server default port
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

/// `OpenAI` API base URL
pub const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";

/// `OpenAI` default embedding model
pub const OPENAI_DEFAULT_MODEL: &str = "text-embedding-3-small";

/// Gemini API base URL
pub const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// Gemini default embedding model
pub const GEMINI_DEFAULT_MODEL: &str = "text-embedding-004";

/// Gemini max tokens per request
pub const GEMINI_MAX_TOKENS: usize = 2048;

/// Ollama default base URL
pub const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Ollama default embedding model
pub const OLLAMA_DEFAULT_MODEL: &str = "nomic-embed-text";

/// Ollama max tokens for limited models (e.g., `all-minilm`)
pub const OLLAMA_MAX_TOKENS_LIMITED: usize = 512;

/// Ollama max tokens default
pub const OLLAMA_MAX_TOKENS_DEFAULT: usize = 8192;

/// Anthropic/Voyage AI API base URL
pub const VOYAGEAI_API_BASE_URL: &str = "https://api.voyageai.com/v1";

/// Anthropic default embedding model
pub const ANTHROPIC_DEFAULT_MODEL: &str = "voyage-3";

/// `VoyageAI` default embedding model
pub const VOYAGEAI_DEFAULT_MODEL: &str = "voyage-code-3";

/// `FastEmbed` default model
pub const FASTEMBED_DEFAULT_MODEL: &str = "AllMiniLML6V2";

/// `FastEmbed` max tokens (approximate)
pub const FASTEMBED_MAX_TOKENS: usize = 512;

/// `FastEmbed` actor channel capacity
pub const FASTEMBED_ACTOR_CHANNEL_CAPACITY: usize = 100;

/// Default HTTP request timeout in seconds
pub const DEFAULT_HTTP_TIMEOUT_SECS: u64 = 30;

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

/// `EdgeVec` HNSW M parameter (max connections per node in layers > 0)
pub const EDGEVEC_HNSW_M: u32 = 16;

/// `EdgeVec` HNSW M0 parameter (max connections per node in layer 0)
pub const EDGEVEC_HNSW_M0: u32 = 32;

/// `EdgeVec` HNSW `ef_construction` parameter
pub const EDGEVEC_HNSW_EF_CONSTRUCTION: u32 = 200;

/// `EdgeVec` HNSW `ef_search` parameter
pub const EDGEVEC_HNSW_EF_SEARCH: u32 = 64;

/// `EdgeVec` default dimensions (for `OpenAI` embeddings)
pub const EDGEVEC_DEFAULT_DIMENSIONS: usize = 1536;

// ============================================================================
// MILVUS VECTOR STORE CONSTANTS
// ============================================================================

/// Milvus field varchar max length
pub const MILVUS_FIELD_VARCHAR_MAX_LENGTH: i32 = 512;

/// Milvus metadata varchar max length
pub const MILVUS_METADATA_VARCHAR_MAX_LENGTH: i32 = 65535;

/// Milvus `IvfFlat` nlist parameter
pub const MILVUS_IVFFLAT_NLIST: u32 = 128;

/// Milvus default port
pub const MILVUS_DEFAULT_PORT: u16 = 19530;

/// Milvus default connection timeout in seconds
pub const MILVUS_DEFAULT_TIMEOUT_SECS: u64 = 10;

/// Milvus default query limit for aggregation queries
pub const MILVUS_DEFAULT_QUERY_LIMIT: i64 = 10_000;

/// Milvus pagination batch size (keeps gRPC responses under 4MB limit)
pub const MILVUS_QUERY_BATCH_SIZE: usize = 100;

/// Milvus flush retry count
pub const MILVUS_FLUSH_RETRY_COUNT: usize = 3;

/// Milvus flush retry backoff in milliseconds
pub const MILVUS_FLUSH_RETRY_BACKOFF_MS: u64 = 1000;

/// Milvus index creation retry backoff in milliseconds
pub const MILVUS_INDEX_RETRY_BACKOFF_MS: u64 = 500;

/// Milvus distance metric type for search
pub const MILVUS_DISTANCE_METRIC: &str = "L2";

/// Milvus vector index name
pub const MILVUS_VECTOR_INDEX_NAME: &str = "vector_index";

// ============================================================================
// QDRANT VECTOR STORE CONSTANTS
// ============================================================================

/// Qdrant default server port
pub const QDRANT_DEFAULT_PORT: u16 = 6333;

/// Qdrant distance metric
pub const QDRANT_DISTANCE_METRIC: &str = "Cosine";

// ============================================================================
// PINECONE VECTOR STORE CONSTANTS
// ============================================================================

/// Pinecone upsert batch size
pub const PINECONE_UPSERT_BATCH_SIZE: usize = 100;

// ============================================================================
// DATABASE CONSTANTS
// ============================================================================

/// File hash tombstone TTL in seconds (30 days)
pub const FILE_HASH_TOMBSTONE_TTL_SECS: i64 = 30 * 24 * 60 * 60;

/// Max characters for SQL statement preview in log messages.
pub const SQL_PREVIEW_CHAR_LIMIT: usize = 120;

// ============================================================================
// HTTP HEADER CONSTANTS
// ============================================================================

/// HTTP Authorization header name.
pub const HTTP_HEADER_AUTHORIZATION: &str = "Authorization";

/// HTTP Content-Type header name.
pub const HTTP_HEADER_CONTENT_TYPE: &str = "Content-Type";

/// Pinecone API key header name.
pub const PINECONE_API_KEY_HEADER: &str = "Api-Key";

// ============================================================================
// EMBEDDING API ENDPOINT PATHS
// ============================================================================

/// OpenAI/Anthropic/VoyageAI embeddings endpoint path.
pub const EMBEDDING_API_ENDPOINT: &str = "/embeddings";

/// Ollama embed API endpoint path.
pub const OLLAMA_EMBED_ENDPOINT: &str = "/api/embed";

/// Embedding operation name for HTTP client calls.
pub const EMBEDDING_OPERATION_NAME: &str = "embeddings";

// ============================================================================
// VECTOR STORE STATS FIELD NAMES
// ============================================================================

/// Stats JSON field: collection name.
pub const STATS_FIELD_COLLECTION: &str = "collection";

/// Stats JSON field: provider name.
pub const STATS_FIELD_PROVIDER: &str = "provider";

/// Stats JSON field: operational status.
pub const STATS_FIELD_STATUS: &str = "status";

/// Stats JSON field: vector count.
pub const STATS_FIELD_VECTORS_COUNT: &str = "vectors_count";

/// Stats JSON field: row count.
pub const STATS_FIELD_ROW_COUNT: &str = "row_count";

/// Status value: active/ready.
pub const STATUS_ACTIVE: &str = "active";

/// Status value: unknown/unavailable.
pub const STATUS_UNKNOWN: &str = "unknown";

// ============================================================================
// MILVUS ERROR PATTERN STRINGS
// ============================================================================

/// Milvus error: collection does not exist (gRPC error message).
pub const MILVUS_ERROR_COLLECTION_NOT_EXISTS: &str = "CollectionNotExists";

/// Milvus error: rate limit exceeded.
pub const MILVUS_ERROR_RATE_LIMIT: &str = "RateLimit";

// ============================================================================
// EDGEVEC CONSTANTS
// ============================================================================

/// `EdgeVec` quantization type for scalar quantization.
pub const EDGEVEC_QUANTIZATION_TYPE: &str = "scalar";

// ============================================================================
// VECTOR STORE METADATA FIELD NAMES
// ============================================================================

/// Vector store field: document identifier.
pub const VECTOR_FIELD_ID: &str = "id";

/// Vector store field: source file path.
pub const VECTOR_FIELD_FILE_PATH: &str = "file_path";

/// Vector store field: start line number.
pub const VECTOR_FIELD_START_LINE: &str = "start_line";

/// Vector store field: line number (legacy/fallback field name).
pub const VECTOR_FIELD_LINE_NUMBER: &str = "line_number";

/// Vector store field: content text.
pub const VECTOR_FIELD_CONTENT: &str = "content";

/// Vector store field: embedding vector.
pub const VECTOR_FIELD_VECTOR: &str = "vector";

/// Vector store field: programming language.
pub const VECTOR_FIELD_LANGUAGE: &str = "language";

/// Vector store field: metadata JSON blob.
pub const VECTOR_FIELD_METADATA: &str = "metadata";

// ============================================================================
// EMBEDDING API FIELD NAMES
// ============================================================================

/// Embedding API response field: embedding vector.
pub const EMBEDDING_RESPONSE_FIELD: &str = "embedding";

/// Embedding API request field: model name.
pub const EMBEDDING_PARAM_MODEL: &str = "model";

/// Embedding API request field: input text.
pub const EMBEDDING_PARAM_INPUT: &str = "input";

// ============================================================================
// MILVUS SCHEMA/INDEX PARAMETER NAMES
// ============================================================================

/// Milvus index parameter: distance metric type.
pub const MILVUS_PARAM_METRIC_TYPE: &str = "metric_type";

/// Milvus index parameter: IVF nlist value.
pub const MILVUS_PARAM_NLIST: &str = "nlist";
