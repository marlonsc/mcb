//! # MCP Context Browser - Provider Implementations
//!
//! This crate contains all user-selectable provider implementations following
//! Clean Architecture principles. Each provider implements a port (trait)
//! defined in `mcb-domain`.
//!
//! **Documentation**: [`docs/modules/providers.md`](../../../docs/modules/providers.md) |
//! **Strategy**: [`ADR-030`](../../../docs/adr/030-multi-provider-strategy.md),
//! [`ADR-003`](../../../docs/adr/003-unified-provider-architecture.md)
//!
//! ## Provider Categories
//!
//! | Category | Port | Implementations |
//! | ---------- | ------ | ----------------- |
//! | Embedding | `EmbeddingProvider` | `OpenAI`, Ollama, `VoyageAI`, Gemini, `FastEmbed` |
//! | Vector Store | `VectorStoreProvider` | `EdgeVec`, Encrypted, Milvus, Pinecone, Qdrant |
//! | Cache | `CacheProvider` | Moka, Redis |
//! | Events | `EventPublisher` | Tokio, Nats |
//! | Hybrid Search | `HybridSearchProvider` | `HybridSearchEngine` |
//! | Language | `LanguageChunkingProvider` | Rust, Python, Go, Java, etc. |
//!
//! ## Feature Flags
//!
//! Each provider can be enabled/disabled via feature flags for minimal builds:
//!
//! ```toml
//! [dependencies]
//! mcb-providers = { version = "0.1", default-features = false, features = ["embedding-ollama", "cache-moka"] }
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use mcb_providers::embedding::OllamaEmbeddingProvider;
//! use mcb_providers::cache::MokaCacheProvider;
//! use mcb_providers::language::RustProcessor;
//! ```

// Allow collapsible_if for complex conditional logic

// Re-export mcb-domain types commonly used with providers
pub use mcb_domain::error::{Error, Result};
pub use mcb_domain::ports::CryptoProvider;
pub use mcb_domain::ports::{
    CacheProvider, ComplexityAnalyzer, DeadCodeDetector, EmbeddingProvider, HybridSearchProvider,
    LanguageChunkingProvider, TdgScorer, VcsProvider, VectorStoreProvider,
};

/// Provider-specific constants
pub mod constants;

/// Common macros for providers layer
/// Shared utilities for provider implementations
pub mod utils;

/// Embedding provider implementations
///
/// Implements `EmbeddingProvider` trait for various embedding APIs.
pub mod embedding;

/// Vector store provider implementations
///
/// Implements `VectorStoreProvider` trait for vector storage backends.
pub mod vector_store;

/// Cache provider implementations
///
/// Implements `CacheProvider` trait for caching backends.
pub mod cache;

/// Native PMAT-style analysis provider implementations.
pub mod analysis;

/// Event publisher implementations (simple EventPublisher trait)
///
/// Implements `EventPublisher` trait for event bus backends.
pub mod events;

/// Language chunking provider implementations
///
/// Implements `LanguageChunkingProvider` trait for AST-based code parsing.
/// Also provides `IntelligentChunker` that implements `CodeChunker` trait.
pub mod language;

/// Hybrid search provider implementations
///
/// Implements `HybridSearchProvider` trait for combined BM25 + semantic search.
/// Provides BM25 text ranking algorithm and hybrid score fusion.
pub mod hybrid_search;

// Re-export hybrid search providers
pub use hybrid_search::HybridSearchEngine;

/// Database providers — SeaORM repositories for structured persistence.
/// Database-agnostic (SQLite + PostgreSQL via connection string).
pub mod database;

pub use database::seaorm::migration;

/// Project type detection providers
pub mod project_detection;

/// Git-related providers for repository operations
///
/// Provides submodule discovery with recursive traversal.
pub mod vcs;
/// Workflow FSM provider for ADR-034
///
/// Implements state machine transitions and session management
pub mod workflow;
