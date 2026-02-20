//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
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

/// Event system and messaging constants.
pub mod events;

/// File system operation constants.
pub mod fs;

/// Health check and monitoring constants.
pub mod health;

/// Syntax highlighting constants.
pub mod highlight;

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

/// DI service display names.
pub mod services;

/// Provider name constants for fallback and resolution.
pub mod providers;
