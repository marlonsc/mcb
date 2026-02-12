//! # MCP Context Browser Server
//!
//! MCP protocol server implementation for semantic code analysis using vector embeddings.
//!
//! For user guides and tutorials, see the [online documentation](https://marlonsc.github.io/mcb/).
//!
//! ## Features
//!
//! - **Semantic Search**: AI-powered code understanding and retrieval using vector embeddings
//! - **Multi-Provider**: Support for OpenAI, Ollama, FastEmbed, VoyageAI, Gemini embedding providers
//! - **Vector Storage**: EdgeVec (local), Milvus, Qdrant, Pinecone
//! - **AST Parsing**: 14 programming languages with tree-sitter based code chunking
//! - **Hybrid Search**: Combines BM25 lexical search with semantic similarity
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcb_server::run;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Run with default config (XDG paths + environment)
//!     // server_mode = false uses config to determine mode
//!     run(None, false).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! This crate implements the transport and protocol layer for the MCP Context Browser.
//! It depends on domain contracts and infrastructure services while remaining independent
//! of specific provider implementations.
//!
//! ## Core Types
//!
//! The most important types for users:
//!
//! | Type | Description |
//! |------|-------------|
//! | [`McpServer`] | Main server struct |
//! | [`McpServerBuilder`] | Builder for server configuration |
//!
//! ## Feature Flags
//!
//! - `fastembed`: Local embeddings via FastEmbed (default)
//! - `edgevec`: Local HNSW vector storage (default)
//! - `milvus`: Milvus vector database support
//! - `edgevec`: EdgeVec in-memory vector store
//! - `redis-cache`: Redis distributed caching
//! - `full`: All features enabled

// Clippy allows for complex patterns in server code

// Documentation configuration for docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]

#[macro_use]
extern crate rocket;

pub mod admin;
pub mod args;
pub mod auth;
pub mod builder;
pub mod constants;
/// Error mapping helpers for MCP-safe responses.
pub mod error_mapping;
pub mod formatter;
/// Shared helper functions for tool handlers.
pub mod handler_helpers;
pub mod handlers;
pub mod hooks;
pub mod init;
pub mod mcp_server;
pub mod session;
/// Internal template engine (Handlebars-only, forked from Rocket contrib).
pub mod templates;
pub mod tools;
pub mod transport;
/// Shared utility functions.
pub mod utils;

// Re-export core types for public API
pub use builder::McpServerBuilder;
pub use init::run;
pub use mcp_server::McpServer;
