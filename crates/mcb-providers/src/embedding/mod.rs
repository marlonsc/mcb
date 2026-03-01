//! Embedding Provider Implementations
//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#embedding-providers)
//!
//! Converts text into dense vector embeddings for semantic search.
//! Each provider offers different tradeoffs between quality, cost, and privacy.
//!
//! ## Available Providers
//!
//! | Provider | Type | Status |
//! | ---------- | ------ | -------- |
//! | OllamaEmbeddingProvider | Local | Complete |
//! | OpenAIEmbeddingProvider | Cloud | Complete |
//! | VoyageAIEmbeddingProvider | Cloud | Complete |
//! | GeminiEmbeddingProvider | Cloud | Complete |
//! | AnthropicEmbeddingProvider | Cloud | Complete (optional) |
//! | FastEmbedProvider | Local ML | Complete (optional) |
//!
//! ## Provider Selection Guide
//!
//! ### Development/Testing
//! - **Default**: Use `FastEmbedProvider` for local testing (no external dependencies)
//!
//! ### Local/Privacy-First
//! - **Ollama**: Local LLM server with embedding models
//! - **FastEmbed**: Pure local ONNX inference (requires `embedding-fastembed` feature)
//!
//! ### Cloud/Production
//! - **OpenAI**: High quality, widely adopted
//! - **VoyageAI**: Optimized for code embeddings
//! - **Gemini**: Google ecosystem integration

mod anthropic;
mod fastembed;
mod gemini;
mod ollama;
mod openai;
mod voyageai;

// Re-export for convenience
pub use anthropic::AnthropicEmbeddingProvider;
pub use fastembed::FastEmbedProvider;
pub use gemini::GeminiEmbeddingProvider;
pub use ollama::OllamaEmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider;
pub use voyageai::VoyageAIEmbeddingProvider;
