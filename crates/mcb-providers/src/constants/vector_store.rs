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

/// `EdgeVec` quantization type for scalar quantization.
pub const EDGEVEC_QUANTIZATION_TYPE: &str = "scalar";

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

/// Milvus error: collection does not exist (gRPC error message).
pub const MILVUS_ERROR_COLLECTION_NOT_EXISTS: &str = "CollectionNotExists";

/// Milvus error: rate limit exceeded.
pub const MILVUS_ERROR_RATE_LIMIT: &str = "RateLimit";

/// Milvus index parameter: distance metric type.
pub const MILVUS_PARAM_METRIC_TYPE: &str = "metric_type";

/// Milvus index parameter: IVF nlist value.
pub const MILVUS_PARAM_NLIST: &str = "nlist";

/// Qdrant default server port
pub const QDRANT_DEFAULT_PORT: u16 = 6333;

/// Qdrant distance metric
pub const QDRANT_DISTANCE_METRIC: &str = "Cosine";

/// Pinecone upsert batch size
pub const PINECONE_UPSERT_BATCH_SIZE: usize = 100;

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
