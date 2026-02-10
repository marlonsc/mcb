//! # Domain Layer
//!
//! Core business logic and domain types for semantic code analysis.
//! Contains only pure domain entities, value objects, and business rules.
#![allow(missing_docs)]
//!
//! ## Architecture
//!
//! | Component | Description |
//! |-----------|-------------|
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
/// External provider port interfaces
pub mod ports;
/// Provider auto-registration registry
pub mod registry;
/// Repository interfaces
pub mod repositories;
/// Generic schema definitions for persistence (backend-agnostic model)
pub mod schema;
/// Common utilities
pub mod utils;
/// Immutable value objects
pub mod value_objects;

// Re-export commonly used types for convenience
pub use constants::*;
pub use entities::*;
pub use error::{Error, Result};
pub use events::{DomainEvent, EventPublisher, ServiceState};
pub use schema::{
    ForeignKeyDef, MemorySchema, MemorySchemaDdlGenerator, ProjectSchema, SchemaDdlGenerator,
    UniqueConstraintDef,
};
pub use utils::{compute_content_hash, project_type, vcs_context};
pub use value_objects::*;
