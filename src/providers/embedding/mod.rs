//! Embedding provider implementations

pub mod null;
pub mod ollama;
pub mod openai;
pub mod voyageai;

// Re-export for convenience
pub use null::NullEmbeddingProvider;
pub use ollama::OllamaEmbeddingProvider;
pub use openai::OpenAIEmbeddingProvider;
pub use voyageai::VoyageAIEmbeddingProvider;