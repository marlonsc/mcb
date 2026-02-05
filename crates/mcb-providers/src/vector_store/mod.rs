//! Vector Store Provider Implementations
//!
//! Provides storage backends for vector embeddings.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! |----------|------|-------------|
//! | InMemoryVectorStoreProvider | Local | In-memory storage (non-persistent) |
//! | EncryptedVectorStoreProvider | Secure | AES-256-GCM encryption wrapper |
//! | FilesystemVectorStore | Local | Persistent filesystem-based storage |
//! | EdgeVecVectorStoreProvider | Embedded | High-performance HNSW vector store |
//! | MilvusVectorStoreProvider | Cloud | Production-scale cloud vector database |
//! | PineconeVectorStoreProvider | Cloud | Pinecone cloud vector database |
//! | QdrantVectorStoreProvider | Cloud | Qdrant vector search engine |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `InMemoryVectorStoreProvider` for in-memory storage
//! - **Development with data**: Use `InMemoryVectorStoreProvider`
//! - **Production with encryption**: Use `EncryptedVectorStoreProvider` wrapper
//! - **Production local storage**: Use `FilesystemVectorStore` for persistent local storage
//! - **High-performance embedded**: Use `EdgeVecVectorStoreProvider` for sub-ms search
//! - **Cloud production**: Use `MilvusVectorStoreProvider` for distributed cloud deployments

/// Shared helpers for vector store providers (DRY)
pub mod helpers;

pub mod edgevec;
pub mod encrypted;
pub mod filesystem;
pub mod in_memory;
pub mod milvus;
pub mod pinecone;
pub mod qdrant;

// Re-export for convenience
pub use edgevec::{
    EdgeVecConfig, EdgeVecVectorStoreProvider, HnswConfig, MetricType, QuantizerConfig,
};
pub use encrypted::EncryptedVectorStoreProvider;
pub use filesystem::{FilesystemVectorStore, FilesystemVectorStoreConfig};
pub use in_memory::InMemoryVectorStoreProvider;
pub use milvus::MilvusVectorStoreProvider;
pub use pinecone::PineconeVectorStoreProvider;
pub use qdrant::QdrantVectorStoreProvider;
