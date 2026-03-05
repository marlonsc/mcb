//! External Provider Ports
//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)

/// Code analysis provider ports.
pub mod analysis;
/// Provider configuration manager ports.
pub mod config_manager;
/// Cryptographic provider ports.
pub mod crypto;
/// Embedding provider ports.
pub mod embedding;
/// HTTP client provider ports.
pub mod http;
/// Hybrid search provider ports.
pub mod hybrid_search;
/// Language-specific chunking provider ports.
pub mod language_chunking;
/// Metrics provider ports.
pub mod metrics;
/// Project detection provider ports.
pub mod project_detection;
/// Version control system provider ports.
pub mod vcs;
/// Vector store provider ports.
pub mod vector_store;

// Re-exports for canonical access via `ports::providers::{...}`
pub use analysis::{AnalysisFinding, CodeAnalyzer};
pub use config_manager::ProviderConfigManagerInterface;
pub use crypto::{CryptoProvider, EncryptedData};
pub use embedding::EmbeddingProvider;
pub use http::{HttpClientConfig, HttpClientProvider};
pub use hybrid_search::{HybridSearchProvider, HybridSearchResult};
pub use language_chunking::LanguageChunkingProvider;
pub use metrics::{MetricLabels, MetricsError, MetricsProvider, MetricsProviderExt, MetricsResult};
pub use project_detection::ProjectDetector;
pub use vcs::VcsProvider;
pub use vector_store::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
