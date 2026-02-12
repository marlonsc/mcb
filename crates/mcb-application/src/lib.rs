//! Application Layer - MCP Context Browser
//!
//! This crate contains the application layer of the MCP Context Browser,
//! implementing use cases and orchestrating business logic according to
//! Clean Architecture principles.
//!
//! ## Architecture
//!
//! The application layer:
//! - Contains use case implementations (application services)
//! - Defines ports (interfaces) for external dependencies
//! - Orchestrates domain entities and services
//! - Has no dependencies on infrastructure or external frameworks
//!
//! ## Use Cases
//!
//! - Code indexing and ingestion
//! - Semantic search operations
//! - Context management
//! - Admin operations
//!
//! ## Decorators
//!
//! SOLID Open/Closed compliant decorators for cross-cutting concerns:
//! - `decorators::InstrumentedEmbeddingProvider`: Adds timing metrics
//!
//! ## Ports (Interfaces)
//!
//! Defines contracts for external dependencies:
//! - `ports::providers::*`: Provider interfaces (Embedding, VectorStore, Cache, etc.)
//! - `domain_services::*`: Use case interfaces
//!
//! ## Dependencies
//!
//! This crate depends only on:
//! - `mcb-domain`: For domain entities, value objects, and core business rules
//! - Pure Rust libraries for async, serialization, etc.

pub mod constants;
pub mod decorators;
pub mod services;
pub mod use_cases;

pub use decorators::*;
pub use use_cases::*;
