//! External Provider Ports
//!
//! Ports for external services and providers that the domain depends on.
//! These interfaces define contracts for embedding providers, vector stores,
//! and other external services.
//!
//! ## Provider Ports
//!
//! | Port | Description |
//! |------|-------------|
//! | [`EmbeddingProvider`] | Text embedding generation services |
//! | [`VectorStoreProvider`] | Vector storage and similarity search |
//! | [`HybridSearchProvider`] | Combined semantic and keyword search |

/// Embedding provider port
pub mod embedding;
/// Hybrid search provider port
pub mod hybrid_search;
/// Vector store provider port
pub mod vector_store;

// Re-export provider ports
pub use embedding::EmbeddingProvider;
pub use hybrid_search::HybridSearchProvider;
pub use vector_store::VectorStoreProvider;
