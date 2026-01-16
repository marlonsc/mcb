//! Provider Implementations
//!
//! Concrete implementations of domain provider ports for external service integration.
//!
//! ## Submodules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`embedding`] | Embedding provider implementations (OpenAI, Ollama, etc.) |
//!
//! ## Provider Pattern
//!
//! All providers implement traits from `mcb_domain::ports::providers`:
//! - `EmbeddingProvider` - Text-to-vector embeddings
//! - `VectorStoreProvider` - Vector storage and search (Phase 1.3)
//!
//! Providers are registered with Shaku DI for injection.

pub mod embedding;

// Re-export embedding providers
pub use embedding::{
    FastEmbedProvider, GeminiEmbeddingProvider, NullEmbeddingProvider, OllamaEmbeddingProvider,
    OpenAIEmbeddingProvider, VoyageAIEmbeddingProvider,
};
