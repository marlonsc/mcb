//! Use Case Implementations — Application-Layer Services
//!
//! This module contains the use case implementations that orchestrate
//! business logic and coordinate between domain entities and external ports.
//!
//! ## Services
//!
//! - [`AgentSessionServiceImpl`] — Agent session lifecycle, tool history, checkpoints
//! - [`ContextServiceImpl`] — Embedding pipeline, vector lifecycle, semantic search
//! - [`IndexingServiceImpl`] — File discovery, language-aware chunking, async indexing
//! - [`MemoryServiceImpl`] — Hybrid storage (FTS + vector), RRF fusion, timeline
//! - [`SearchServiceImpl`] — Semantic search with application-level filtering
//!
//! ## Dependency Injection
//!
//! All use cases receive their dependencies through constructor injection.
//! They are wired via linkme-based service registries.

pub mod agent_session_service;
pub mod context_service;
pub mod highlight_service;
pub mod indexing_service;
pub mod memory_service;
pub mod search_service;

pub use agent_session_service::*;
pub use context_service::*;
pub use indexing_service::*;
pub use memory_service::*;
pub use search_service::*;
