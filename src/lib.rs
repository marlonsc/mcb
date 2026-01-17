//! # MCP Context Browser
//!
//! A Model Context Protocol server for semantic code analysis using vector embeddings.
//!
//! For user guides and tutorials, see the [online documentation](https://marlonsc.github.io/mcp-context-browser/).
//!
//! ## Features
//!
//! - **Semantic Search**: AI-powered code understanding and retrieval using vector embeddings
//! - **Multi-Provider**: Support for OpenAI, Ollama, FastEmbed, VoyageAI, Gemini embedding providers
//! - **Vector Storage**: Milvus, EdgeVec, or filesystem-based vector storage
//! - **AST Parsing**: 14 programming languages with tree-sitter based code chunking
//! - **Hybrid Search**: Combines BM25 lexical search with semantic similarity
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcp_context_browser::run_server;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Run with default config (XDG paths + environment)
//!     run_server(None).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate follows Clean Architecture with four layers:
//!
//! - **`domain`**: Business entities, value objects, and port traits
//! - **`application`**: Use cases and service orchestration
//! - **`adapters`**: External service implementations (providers, databases)
//! - **`infrastructure`**: Cross-cutting concerns (cache, auth, config, metrics)
//!
//! ## Modules
//!
//! - [`server`]: MCP protocol implementation and handlers
//! - [`domain`]: Core types ([`CodeChunk`], [`Embedding`], [`Language`]) and port traits
//! - [`adapters`]: Embedding and vector store provider implementations
//! - [`application`]: Business logic services (indexing, search, context)
//! - [`infrastructure`]: Cross-cutting concerns (cache, auth, config)
//!
//! ## Core Types
//!
//! The most important types for users:
//!
//! | Type | Description |
//! |------|-------------|
//! | [`McpServer`] | Main server struct |
//! | [`McpServerBuilder`] | Builder for server configuration |
//! | [`CodeChunk`] | A semantically meaningful code segment |
//! | [`Embedding`] | Vector representation of text |
//! | [`Language`] | Supported programming languages |
//! | [`Error`] | Domain error type |
//!
//! ## Feature Flags
//!
//! - `fastembed`: Local embeddings via FastEmbed (default)
//! - `filesystem-store`: Local filesystem vector storage (default)
//! - `milvus`: Milvus vector database support
//! - `edgevec`: EdgeVec in-memory vector store
//! - `redis-cache`: Redis distributed caching
//! - `full`: All features enabled

// Documentation configuration for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Current version of MCP Context Browser
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod adapters;
pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod server;

// Re-export core types for public API
pub use domain::error::{Error, Result};
pub use domain::types::*;

// Re-export main entry points
pub use server::builder::McpServerBuilder;
pub use server::init::run_server;
pub use server::mcp_server::McpServer;
