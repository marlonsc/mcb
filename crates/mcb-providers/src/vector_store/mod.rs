//! Vector Store Provider Implementations
//!
//! **Documentation**: [docs/modules/providers.md](../../../../docs/modules/providers.md#vector-store-providers)
//!
//! Provides storage backends for vector embeddings.
//!
//! ## Available Providers
//!
//! | Provider | Type | Description |
//! | ---------- | ------ | ------------- |
//! | EdgeVecVectorStoreProvider | Embedded | High-performance HNSW vector store (local) |
//! | EncryptedVectorStoreProvider | Secure | AES-256-GCM encryption wrapper |
//! | MilvusVectorStoreProvider | Cloud | Production-scale cloud vector database |
//! | PineconeVectorStoreProvider | Cloud | Pinecone cloud vector database |
//! | QdrantVectorStoreProvider | Cloud | Qdrant vector search engine |
//!
//! ## Provider Selection Guide
//!
//! - **Development/Testing**: Use `EdgeVecVectorStoreProvider` for local HNSW storage
//! - **Production with encryption**: Use `EncryptedVectorStoreProvider` wrapper
//! - **Cloud production**: Use `MilvusVectorStoreProvider` or `QdrantVectorStoreProvider`

pub mod edgevec;
pub mod encrypted;
pub mod milvus;
pub mod pinecone;
pub mod qdrant;

// Re-export for convenience
pub use edgevec::{
    EdgeVecConfig, EdgeVecVectorStoreProvider, HnswConfig, MetricType, QuantizerConfig,
};
pub use encrypted::EncryptedVectorStoreProvider;
pub use milvus::MilvusVectorStoreProvider;
pub use pinecone::PineconeVectorStoreProvider;
pub use qdrant::QdrantVectorStoreProvider;
