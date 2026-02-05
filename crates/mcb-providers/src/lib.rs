//! # MCP Context Browser - Provider Implementations
//!
//! This crate contains all user-selectable provider implementations following
//! Clean Architecture principles. Each provider implements a port (trait)
//! defined in `mcb-domain`.
//!
//! ## Provider Categories
//!
//! | Category | Port | Implementations |
//! |----------|------|-----------------|
//! | Embedding | `EmbeddingProvider` | OpenAI, Ollama, VoyageAI, Gemini, FastEmbed |
//! | Vector Store | `VectorStoreProvider` | InMemory, Encrypted, EdgeVec, Filesystem, Milvus |
//! | Cache | `CacheProvider` | Moka, Redis |
//! | Events | `EventPublisher` | Tokio, Nats, Null |
//! | Hybrid Search | `HybridSearchProvider` | HybridSearchEngine, Null |
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
pub use mcb_domain::ports::providers::{
    CacheProvider, EmbeddingProvider, HybridSearchProvider, LanguageChunkingProvider, VcsProvider,
    VectorStoreProvider,
};

// Re-export CryptoProvider from domain (for encrypted vector store)
pub use mcb_domain::ports::providers::{CryptoProvider, EncryptedData};

/// Provider-specific constants
pub mod constants;

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

/// Event publisher implementations (simple EventPublisher trait)
///
/// Implements `EventPublisher` trait for event bus backends.
pub mod events;

/// HTTP client abstractions
///
/// Provides `HttpClientProvider` trait and configuration for API-based providers.
pub mod http;

/// Code chunking provider implementations
///
/// Implements `CodeChunker` trait for intelligent code chunking.
/// Provides `IntelligentChunker` using tree-sitter and language-specific processors.
pub mod chunking;

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
pub use hybrid_search::{HybridSearchEngine, NullHybridSearchProvider};

/// Database providers (memory repository backends)
///
/// Each backend (SQLite, PostgreSQL, MySQL) has its own submodule and
/// implements the generic schema DDL in its dialect.
pub mod database;

pub use database::{SqliteMemoryDdlGenerator, SqliteSchemaDdlGenerator};

/// Git-related providers for repository operations
///
/// Provides project type detection (Cargo, npm, Python, Go, Maven) and
/// submodule discovery with recursive traversal.
pub mod git;
/// Workflow FSM provider for ADR-034
///
/// Implements state machine transitions and session management
pub mod workflow;
