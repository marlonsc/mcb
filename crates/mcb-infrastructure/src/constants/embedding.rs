/// Vector embedding dimension for the null provider (fallback/default).
pub const EMBEDDING_DIMENSION_NULL: usize = 384;

/// Vector embedding dimension for FastEmbed provider (default model).
pub const EMBEDDING_DIMENSION_FASTEMBED_DEFAULT: usize = 384;

/// Vector embedding dimension for OpenAI small embedding model.
pub const EMBEDDING_DIMENSION_OPENAI_SMALL: usize = 1536;

/// Vector embedding dimension for OpenAI large embedding model.
pub const EMBEDDING_DIMENSION_OPENAI_LARGE: usize = 3072;

/// Vector embedding dimension for OpenAI Ada embedding model.
pub const EMBEDDING_DIMENSION_OPENAI_ADA: usize = 1536;

/// Vector embedding dimension for VoyageAI default embedding model.
pub const EMBEDDING_DIMENSION_VOYAGEAI_DEFAULT: usize = 1024;

/// Vector embedding dimension for VoyageAI code-specific embedding model.
pub const EMBEDDING_DIMENSION_VOYAGEAI_CODE: usize = 1024;

/// Vector embedding dimension for Ollama Nomic embedding model.
pub const EMBEDDING_DIMENSION_OLLAMA_NOMIC: usize = 768;

/// Vector embedding dimension for Ollama MiniLM embedding model.
pub const EMBEDDING_DIMENSION_OLLAMA_MINILM: usize = 384;

/// Vector embedding dimension for Ollama MXBAI embedding model.
pub const EMBEDDING_DIMENSION_OLLAMA_MXBAI: usize = 1024;

/// Vector embedding dimension for Ollama Arctic embedding model.
pub const EMBEDDING_DIMENSION_OLLAMA_ARCTIC: usize = 768;

/// Vector embedding dimension for Ollama default embedding model.
pub const EMBEDDING_DIMENSION_OLLAMA_DEFAULT: usize = 768;

/// Vector embedding dimension for Google Gemini embedding model.
pub const EMBEDDING_DIMENSION_GEMINI: usize = 768;

/// Default vector embedding dimension used when provider-specific dimension is unavailable.
pub const EMBEDDING_DIMENSION_DEFAULT: usize = 512;

/// Maximum input tokens allowed for VoyageAI embedding requests.
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// Maximum output tokens allowed for VoyageAI embedding responses.
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// Time-to-live (TTL) in seconds for OpenAI token cache entries.
pub const OPENAI_TOKEN_CACHE_TTL_SECS: u64 = 7200;

/// Maximum number of tokens allowed per OpenAI embedding request.
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Provider identifier string for OpenAI embedding service.
pub const PROVIDER_OPENAI: &str = "openai";

/// Provider identifier string for EdgeVec vector store.
pub const PROVIDER_EDGEVEC: &str = "edgevec";

/// Model identifier string for OpenAI Ada embedding model.
pub const MODEL_OPENAI_ADA: &str = "text-embedding-ada-002";
