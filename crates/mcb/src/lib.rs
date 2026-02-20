//! # MCP Context Browser
//!
//! A Model Context Protocol server for semantic code analysis using vector embeddings.
//!
//! **Documentation**: [`docs/modules/api-surface.md`](../../../../docs/modules/api-surface.md) |
//! **Architecture**: [`ADR-001`](../../../../docs/adr/001-modular-crates-architecture.md),
//! [`ADR-013`](../../../../docs/adr/013-clean-architecture-crate-separation.md)
//!
//! This crate is the public facade — it re-exports domain types and the server entry point.
//!
//! ## Features
//!
//! - **Semantic Code Search**: Find code by meaning using vector embeddings
//! - **Multi-Language Support**: AST-based parsing for 12+ programming languages
//! - **Multiple Vector Stores**: Support for various vector database backends
//! - **Clean Architecture**: Domain-driven design with dependency injection
//!
//! ## Example
//!
//! ```rust
//! use mcb::entities::CodeChunk;
//!
//! // Domain types are available through the mcb facade
//! let chunk = CodeChunk {
//!     id: "chunk-1".to_string(),
//!     content: "fn main() {}".to_string(),
//!     file_path: "example.rs".to_string(),
//!     start_line: 1,
//!     end_line: 1,
//!     language: "rust".to_string(),
//!     metadata: serde_json::json!({}),
//! };
//! assert_eq!(chunk.id, "chunk-1");
//! ```
//!
//! ## Architecture
//!
//! The codebase follows Clean Architecture principles:
//!
//! - `mcb-domain` — Core business logic, port traits, entities
//! - `mcb-application` — Application services and use cases
//! - `mcb-infrastructure` — DI, config, routing, cross-cutting concerns
//! - `mcb-providers` — Repository + external provider implementations
//! - `mcb-server` — MCP protocol server, admin API endpoints

// CLI module
pub mod cli;

/// Domain layer - core business logic and types
///
/// Re-exports from the domain crate for convenience
pub mod domain {
    pub use mcb_domain::*;
}

/// Server layer - MCP protocol server and handlers
///
/// Re-exports from the server crate for convenience
pub mod server {
    pub use mcb_server::*;
}

/// Infrastructure layer - DI, config, and infrastructure services
///
/// Re-exports from the infrastructure crate for convenience
pub mod infrastructure {
    pub use mcb_infrastructure::*;
}

// Re-export commonly used domain types at the crate root
pub use domain::*;
// Re-export main entry point at the crate root
pub use server::run;
// Re-export server types for convenience
pub use server::{McpServer, McpServerBuilder};
