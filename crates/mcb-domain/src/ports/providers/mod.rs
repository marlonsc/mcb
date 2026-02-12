//! External Provider Ports
//!
//! Ports for external services and providers that the domain depends on.
//! These interfaces define contracts for embedding providers, vector stores,
//! language chunking, caching, cryptography, and other external services.
//!
//! ## Provider Ports
//!
//! | Port | Description |
//! | ------ | ------------- |
//! | EmbeddingProvider | Text embedding generation services |
//! | VectorStoreProvider | Vector storage and similarity search |
//! | VectorStoreBrowser | Collection and file browsing for Admin UI |
//! | HybridSearchProvider | Combined semantic and keyword search |
//! | LanguageChunkingProvider | Language-specific code chunking |
//! | MetricsAnalysisProvider | Code complexity metrics analysis |
//! | ValidationProvider | Pluggable code validation engines |
//! | CacheProvider | Caching backend services |
//! | CryptoProvider | Encryption/decryption services |
//! | ProjectDetector | Project type detection (Cargo, npm, Python, Go, Maven) |

/// Native PMAT-style analysis provider ports
pub mod analysis;
/// Cache provider port
pub mod cache;
/// Config provider port
pub mod config;
/// Crypto provider port
pub mod crypto;
/// Embedding provider port
pub mod embedding;
/// HTTP client provider port
pub mod http;
/// Hybrid search provider port
pub mod hybrid_search;
/// Language chunking provider port
mod language_chunking;
/// Observability metrics provider port (Prometheus/OpenTelemetry)
pub mod metrics;
/// Code metrics analysis provider port
pub mod metrics_analysis;
/// Project detection provider port
pub mod project_detection;
/// Validation provider port
pub mod validation;
/// Version control system provider port
pub mod vcs;
/// Vector store provider port
pub mod vector_store;

// Re-export provider ports for convenience
pub use analysis::{
    ComplexityAnalyzer, ComplexityFinding, DeadCodeDetector, DeadCodeFinding, TdgFinding, TdgScorer,
};
pub use cache::{CacheEntryConfig, CacheProvider, CacheProviderFactoryInterface, CacheStats};
pub use config::ProviderConfigManagerInterface;
pub use crypto::{CryptoProvider, EncryptedData};
pub use embedding::EmbeddingProvider;
pub use http::{HttpClientConfig, HttpClientProvider};
pub use hybrid_search::{HybridSearchProvider, HybridSearchResult};
pub use language_chunking::LanguageChunkingProvider;
pub use metrics::{MetricLabels, MetricsError, MetricsProvider, MetricsResult};
pub use metrics_analysis::{
    FileMetrics, FunctionMetrics, HalsteadMetrics, MetricsAnalysisProvider,
};
pub use project_detection::{ProjectDetector, ProjectDetectorConfig, ProjectDetectorEntry};
pub use validation::{ValidationOptions, ValidationProvider, ValidatorInfo};
pub use vcs::VcsProvider;
pub use vector_store::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
