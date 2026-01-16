//! Embedding Provider Implementations
//!
//! Converts text into dense vector embeddings for semantic search.
//! Each provider offers different tradeoffs between quality, cost, and privacy.
//!
//! ## Available Providers
//!
//! | Provider | Dimensions | Status |
//! |----------|-----------|--------|
//! | [`NullEmbeddingProvider`] | 384 | Complete - Testing/stubs |
//!
//! ## Planned Providers (Phase 1.2 completion)
//!
//! - OpenAI (text-embedding-3)
//! - Ollama (local models)
//! - VoyageAI (code-optimized)
//! - Gemini (Google ecosystem)
//! - FastEmbed (local, default)
//!
//! ## Provider Selection Guide
//!
//! ### Development/Testing
//! - **Default**: Use NullEmbeddingProvider for unit tests
//!
//! ### Production
//! - Cloud: OpenAI or VoyageAI
//! - Privacy: Ollama or FastEmbed

pub mod fastembed;
pub mod gemini;
pub mod helpers;
pub mod null;
pub mod ollama;
pub mod openai;
pub mod voyageai;

// Re-export for convenience
pub use fastembed::FastEmbedProvider;
pub use gemini::GeminiEmbeddingProvider;
pub use helpers::constructor;
pub use null::NullEmbeddingProvider;
pub use ollama::OllamaEmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider;
pub use voyageai::VoyageAIEmbeddingProvider;
