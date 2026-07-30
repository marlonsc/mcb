//! Memory Service Use Case
//!
//! # Overview
//! The `MemoryService` implements a comprehensive system for storing, retrieving, and analyzing
//! observations and long-term memory. It acts as the "brain" of the system, allowing agents
//! to recall past context, decisions, and error patterns.
//!
//! # Responsibilities
//! - **Hybrid Storage**: Persisting observations in both a relational DB (`SQLite`) for metadata/FTS
//!   and a Vector Store for semantic similarity.
//! - **Hybrid Search**: Combining keyword-based (FTS) and semantic (Vector) search results using
//!   Reciprocal Rank Fusion (RRF) for high-quality recall.
//! - **Timeline Management**: Retrieving observations in chronological order to reconstruct context.
//! - **Pattern Recognition**: Storing and retrieving error patterns to avoid repeating mistakes.
//! - **Session Summarization**: Compiling and storing high-level summaries of agent sessions.
//!
//! # Architecture
//! Implements `MemoryServiceInterface` and coordinates:
//! - `MemoryRepository`: For precise storage and FTS.
//! - `VectorStoreProvider`: For fuzzy semantic search.
//! - `EmbeddingProvider`: For generating vector representations of memory content.

mod helpers;
mod interface;
mod observation;
mod registry;
mod search;
mod service;
mod session;

pub use service::MemoryServiceImpl;
