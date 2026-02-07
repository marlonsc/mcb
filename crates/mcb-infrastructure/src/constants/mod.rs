//! Infrastructure layer constants. Domain-specific constants are in `mcb_domain::constants`.

/// Abstract syntax tree (AST) related constants.
pub mod ast;

/// Authentication and authorization constants.
pub mod auth;

/// Cache configuration and behavior constants.
pub mod cache;

/// Configuration and environment constants.
pub mod config;

/// Cryptographic operation constants.
pub mod crypto;

/// Database connection and operation constants.
pub mod db;

/// Vector embedding provider and dimension constants.
pub mod embedding;

/// Error message templates and constants.
pub mod error_msgs;

/// Event system and messaging constants.
pub mod events;

/// File system operation constants.
pub mod fs;

/// Health check and monitoring constants.
pub mod health;

/// HTTP server and client constants.
pub mod http;

/// Programming language and syntax constants.
pub mod lang;

/// System limits and constraints.
pub mod limits;

/// Logging configuration and format constants.
pub mod logging;

/// Metadata and annotation constants.
pub mod metadata;

/// Metrics collection and reporting constants.
pub mod metrics;

/// Network communication constants.
pub mod network;

/// Operations and workflow constants.
pub mod ops;

/// Process management constants.
pub mod process;

/// Resilience and fault tolerance constants.
pub mod resilience;

/// Search and indexing constants.
pub mod search;

/// Synchronization and concurrency constants.
pub mod sync;

// Re-export common constants for backward compatibility
pub use ast::*;
pub use auth::*;
pub use cache::*;
pub use config::*;
pub use crypto::*;
pub use db::*;
pub use embedding::*;
pub use error_msgs::*;
pub use events::*;
pub use fs::*;
pub use health::*;
pub use http::*;
pub use lang::*;
pub use limits::*;
pub use logging::*;
pub use metadata::*;
pub use metrics::*;
pub use network::*;
pub use ops::*;
pub use process::*;
pub use resilience::*;
pub use search::*;
pub use sync::*;

// Re-export domain constants for convenience
pub use mcb_domain::constants::*;
