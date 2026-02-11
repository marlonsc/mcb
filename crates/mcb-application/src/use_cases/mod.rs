//! Use Cases - Application Layer Services
//!
//! This module contains the use case implementations that orchestrate
//! business logic and coordinate between domain entities and external ports.
//!
//! ## Use Cases Implemented
//!
//! - `context_service`: Code intelligence and semantic operations
//! - `search_service`: Semantic search operations
//! - `indexing_service`: Code indexing and ingestion operations
//!
//! ## Dependency Injection
//!
//! All use cases are designed to work with dependency injection via dill IoC.
//! They receive their dependencies (ports) through constructor injection.

pub mod agent_session_service;
pub mod context_service;
pub mod indexing_service;
pub mod memory_service;
pub mod search_service;
pub mod validation_service;
pub mod vcs_indexing;

pub use agent_session_service::*;
pub use context_service::*;
pub use indexing_service::*;
pub use memory_service::*;
pub use search_service::*;
pub use validation_service::*;
pub use vcs_indexing::*;
