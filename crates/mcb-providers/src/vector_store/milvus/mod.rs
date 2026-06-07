//! Milvus vector store provider implementation
//!
//! High-performance cloud vector database using Milvus.
//! Supports production-scale vector storage with automatic indexing and distributed search.

/// Milvus admin operations (create, drop, health).
pub mod admin;
/// Milvus collection browsing operations.
pub mod browser;
mod client;
mod helpers;
mod list;
mod provider;
mod registry;
/// Schema utilities for Milvus collections.
pub mod schema;
mod search;

pub use client::{MilvusVectorStoreProvider, to_milvus_name};

// Re-export internal types for sibling modules that use `super::*`
use client::*;
use mcb_domain::error::Result;
