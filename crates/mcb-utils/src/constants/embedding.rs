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
