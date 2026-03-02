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
//! | [`utils`] | Shared utilities and helpers |
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
/// MCP JSON-RPC protocol types (domain-level contract)
pub mod protocol;
/// Provider auto-registration registry
pub mod registry;
/// Common utilities
pub mod utils;
/// Immutable value objects
pub mod value_objects;

// Re-export commonly used types for convenience
pub use entities::*;
pub use error::{Error, Result};
pub use events::{DomainEvent, EventPublisher, ServiceState};
pub use value_objects::*;

// ── Test utilities: crate-level re-exports ─────────────────────────────
// Enables canonical paths: `mcb_domain::test_utils`, `mcb_domain::test_collection`, etc.
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::assertions as test_assertions;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::chunk_fixtures as test_chunk_fixtures;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::collection as test_collection;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::fixtures as test_fixtures;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::fs_scan as test_fs_scan;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::git_helpers as test_git_helpers;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::guards as test_guards;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::json_helpers as test_json_helpers;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::mcp_assertions as test_mcp_assertions;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::search_fixtures as test_search_fixtures;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::service_detection as test_service_detection;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::services_config as test_services_config;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::sync_helpers as test_sync_helpers;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::timeouts as test_timeouts;
#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::utils as test_utils;

#[cfg(any(test, feature = "test-utils"))]
pub use utils::tests::http_mcp as test_http_mcp;
