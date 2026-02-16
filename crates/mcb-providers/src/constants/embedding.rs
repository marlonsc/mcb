/// `VoyageAI` max input tokens
pub const VOYAGEAI_MAX_INPUT_TOKENS: usize = 16000;

/// `VoyageAI` max output tokens
pub const VOYAGEAI_MAX_OUTPUT_TOKENS: usize = 16000;

/// `OpenAI` max tokens per request
pub const OPENAI_MAX_TOKENS_PER_REQUEST: usize = 8191;

/// Anthropic (Voyage AI) max input tokens per request
pub const ANTHROPIC_MAX_INPUT_TOKENS: usize = 32000;

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

/// Ollama server default port
pub const OLLAMA_DEFAULT_PORT: u16 = 11434;

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

/// OpenAI/Anthropic/VoyageAI embeddings endpoint path.
pub const EMBEDDING_API_ENDPOINT: &str = "/embeddings";

/// Ollama embed API endpoint path.
pub const OLLAMA_EMBED_ENDPOINT: &str = "/api/embed";

/// Embedding operation name for HTTP client calls.
pub const EMBEDDING_OPERATION_NAME: &str = "embeddings";

/// Embedding API response field: embedding vector.
pub const EMBEDDING_RESPONSE_FIELD: &str = "embedding";

/// Embedding API request field: model name.
pub const EMBEDDING_PARAM_MODEL: &str = "model";

/// Embedding API request field: input text.
pub const EMBEDDING_PARAM_INPUT: &str = "input";
