//! Use Cases - Application Layer Services
//!
//! **Documentation**: [docs/modules/application.md](../../../../docs/modules/application.md#use-cases)
//!
//! This module contains the use case implementations that orchestrate
//! business logic and coordinate between domain entities and external ports.
//!
//! ## Use Cases Implemented
//!
//! - `agent_session_service`: Manages agent session lifecycle and tool history
//! - `context_service`: Code intelligence and semantic operations
//! - `indexing_service`: Code indexing and ingestion operations
//! - `memory_service`: Observation/memory capture and awareness
//! - `search_service`: Semantic, hybrid, and lexical search operations
//!
//! ## Dependency Injection
//!
//! All use cases receive their dependencies through constructor injection.
//! They receive their dependencies (ports) through constructor injection.

pub mod agent_session_service;
pub mod context_service;
pub mod indexing_service;
pub mod memory_service;
pub mod search_service;

pub use agent_session_service::*;
pub use context_service::*;
pub use indexing_service::*;
pub use memory_service::*;
pub use search_service::*;
