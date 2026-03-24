//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
/// Null/placeholder provider dimension (MiniLM-L6 default).
pub const EMBEDDING_DIMENSION_NULL: usize = 384;
/// `FastEmbed` default model dimension (MiniLM-L6-v2).
pub const EMBEDDING_DIMENSION_FASTEMBED_DEFAULT: usize = 384;
/// `OpenAI` text-embedding-3-small dimension.
pub const EMBEDDING_DIMENSION_OPENAI_SMALL: usize = 1536;
/// `OpenAI` text-embedding-3-large dimension.
pub const EMBEDDING_DIMENSION_OPENAI_LARGE: usize = 3072;
/// `OpenAI` text-embedding-ada-002 dimension.
pub const EMBEDDING_DIMENSION_OPENAI_ADA: usize = 1536;
/// Voyage AI default model dimension.
pub const EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT: usize = 1024;
/// Voyage AI code-specialized model dimension.
pub const EMBEDDING_DIMENSION_VOYAGEAI_CODE: usize = 1024;
/// Ollama nomic-embed-text dimension.
pub const EMBEDDING_DIMENSION_OLLAMA_NOMIC: usize = 768;
/// Ollama all-minilm dimension.
pub const EMBEDDING_DIMENSION_OLLAMA_MINILM: usize = 384;
/// Ollama mxbai-embed-large dimension.
pub const EMBEDDING_DIMENSION_OLLAMA_MXBAI: usize = 1024;
/// Ollama snowflake-arctic-embed dimension.
pub const EMBEDDING_DIMENSION_OLLAMA_ARCTIC: usize = 768;
/// Ollama fallback default dimension.
pub const EMBEDDING_DIMENSION_OLLAMA_DEFAULT: usize = 768;
/// Anthropic default embedding dimension.
pub const EMBEDDING_DIMENSION_ANTHROPIC_DEFAULT: usize = 1024;
/// Anthropic lite embedding dimension.
pub const EMBEDDING_DIMENSION_ANTHROPIC_LITE: usize = 512;
/// Anthropic code-specialized embedding dimension.
pub const EMBEDDING_DIMENSION_ANTHROPIC_CODE: usize = 1024;
/// Google Gemini embedding dimension.
pub const EMBEDDING_DIMENSION_GEMINI: usize = 768;
/// System-wide default embedding dimension.
pub const EMBEDDING_DIMENSION_DEFAULT: usize = 512;

// ============================================================================
// Provider API Configuration
// ============================================================================

/// `VoyageAI` max input tokens.
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// `OpenAI` max tokens per request.
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Anthropic (Voyage AI) max input tokens per request.
pub const ANTHROPIC_MAX_INPUT_TOKENS: usize = 32000;

/// `OpenAI` API base URL.
pub const OPENAI_API_BASE_URL: &str = "https://api.openai.com/v1";

/// Gemini API base URL.
pub const GEMINI_API_BASE_URL: &str = "https://generativelanguage.googleapis.com";

/// Gemini max tokens per request.
pub const GEMINI_MAX_TOKENS: usize = 2048;

/// Ollama server default port.
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

/// Ollama default base URL.
pub const OLLAMA_DEFAULT_BASE_URL: &str = "http://localhost:11434";

/// Ollama default embedding model.
pub const OLLAMA_DEFAULT_MODEL: &str = "nomic-embed-text";

/// Ollama max tokens for limited models (e.g., `all-minilm`).
pub const OLLAMA_MAX_TOKENS_LIMITED: usize = 512;

/// Ollama max tokens default.
pub const OLLAMA_MAX_TOKENS_DEFAULT: usize = 8192;

/// Anthropic/Voyage AI API base URL.
pub const VOYAGEAI_API_BASE_URL: &str = "https://api.voyageai.com/v1";

/// `FastEmbed` default model.
pub const FASTEMBED_DEFAULT_MODEL: &str = "AllMiniLML6V2";

/// `FastEmbed` max tokens (approximate).
pub const FASTEMBED_MAX_TOKENS: usize = 512;

/// `FastEmbed` actor channel capacity.
pub const FASTEMBED_ACTOR_CHANNEL_CAPACITY: usize = 100;

// ============================================================================
// Embedding API Field Names
// ============================================================================

/// OpenAI/Anthropic/VoyageAI embeddings endpoint path.
pub const EMBEDDING_API_ENDPOINT: &str = "/embeddings";

/// Embedding operation name for HTTP client calls.
pub const EMBEDDING_OPERATION_NAME: &str = "embeddings";

/// Embedding API response field: embedding vector.
pub const EMBEDDING_RESPONSE_FIELD: &str = "embedding";

/// Embedding API request field: model name.
pub const EMBEDDING_PARAM_MODEL: &str = "model";

/// Embedding API request field: input text.
pub const EMBEDDING_PARAM_INPUT: &str = "input";
