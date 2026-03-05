//! `EdgeVec` Vector Store Provider
//!
//! High-performance embedded vector database implementation using `EdgeVec`.
//! `EdgeVec` provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

mod actor;
mod client;
pub mod config;
mod provider;
mod registry;

pub use client::EdgeVecVectorStoreProvider;
pub use config::{EdgeVecConfig, HnswConfig, MetricType, QuantizerConfig};

// Re-export internal types for sibling modules that use `super::*`
pub(self) use client::*;
