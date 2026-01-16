//! Adapter Implementations
//!
//! Concrete implementations of domain ports for external service integration.
//! Adapters bridge the domain layer with infrastructure concerns like databases,
//! external APIs, and storage systems.
//!
//! ## Submodules
//!
//! | Module | Description |
//! |--------|-------------|
//! | [`http_client`] | HTTP client infrastructure for API-based providers |
//! | [`providers`] | Provider implementations (embedding, vector store) |
//! | [`repository`] | Repository implementations for data persistence |
//!
//! ## Adding New Adapters
//!
//! To add a new adapter:
//! 1. Create a new module in the appropriate subdirectory
//! 2. Implement the domain port trait from `mcb_domain::ports`
//! 3. Add Shaku component annotations for DI
//! 4. Re-export from this module

pub mod http_client;
pub mod providers;
pub mod repository;

// Re-export HTTP client infrastructure
pub use http_client::{HttpClientConfig, HttpClientPool, HttpClientProvider, SharedHttpClient};

// Re-export embedding providers
pub use providers::{
    FastEmbedProvider, GeminiEmbeddingProvider, NullEmbeddingProvider, OllamaEmbeddingProvider,
    OpenAIEmbeddingProvider, VoyageAIEmbeddingProvider,
};

// Re-export repository implementations
pub use repository::{VectorStoreChunkRepository, VectorStoreSearchRepository};
