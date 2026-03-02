//! # MCP Context Browser - Domain Layer
//!
//! **Documentation**: [`docs/modules/domain.md`](../../../docs/modules/domain.md)
//!
//! Canonical business logic and domain entities.
//! This crate is the single source of truth for domain concepts.
//!
//! **Strategy**: [`ADR-013`](../../../docs/adr/013-clean-architecture-crate-separation.md)
//!
//! ## Architecture
//!
//! | Component | Description |
//! | ----------- | ------------- |
//! | [`entities`] | Core business entities with identity |
//! | [`value_objects`] | Immutable value objects |
//! | [`ports`] | External provider port interfaces |
//! | [`constants`] | Domain constants |
//! | [`error`] | Domain error types |
//!
//! ## Clean Architecture Principles
//!
//! - **Entities** are at the center with business rules
//! - **Value Objects** are immutable and compared by value
//! - **No external dependencies** - only standard library and core traits
//! - **Pure business logic** - no infrastructure or application concerns
//!
//! ## Focus: Architecture & Integration
//!
//! For developers (and agents) looking to integrate with MCB's dependency system:
//!
//! - **Static DI / Containerless Architecture (CA)**: See [`registry`] for the
//!   `linkme`-based registration backbone. This is where providers are "linked" to the domain.
//! - **Opaque DI Context**: See [`registry::ServiceResolutionContext`]. It carries
//!   infrastructure dependencies (DB, Config) through the domain layer without creating
//!   cyclic dependencies.
//! - **Test Utilities**: See [`utils::tests`] for the centralized testing
//!   scaffolding, including Golden Tests, Invariant Assertions, and DI-ready fixtures.
//!
//! ## Example
//!
//! ```
//! use mcb_domain::entities::CodeChunk;
//! use mcb_domain::value_objects::Embedding;
//!
//! // Create a code chunk entity
//! let chunk = CodeChunk {
//!     id: "chunk-1".to_string(),
//!     content: "fn main() {}".to_string(),
//!     file_path: "example.rs".to_string(),
//!     start_line: 1,
//!     end_line: 1,
//!     language: "rust".to_string(),
//!     metadata: serde_json::json!({}),
//! };
//!
//! // Create an embedding value object
//! let embedding = Embedding { vector: vec![0.1, 0.2], model: "test".into(), dimensions: 2 };
//! ```

/// Common macros
#[macro_use]
pub mod macros;

/// Domain-level constants
pub mod constants;
/// Core business entities with identity
pub mod entities;
/// Domain error types
pub mod error;
/// Domain event interfaces
pub mod events;
/// Domain surface for infrastructure (plug points; infra registers at startup).
pub mod infra;
/// External provider port interfaces
pub mod ports;
/// Provider auto-registration registry
pub mod registry;
/// Common utilities
pub mod utils;
/// Immutable value objects
pub mod value_objects;

// Re-export commonly used types for convenience
pub use constants::values::*;
pub use entities::*;
pub use error::{Error, Result};
pub use events::{DomainEvent, EventPublisher, ServiceState};
pub use value_objects::*;
