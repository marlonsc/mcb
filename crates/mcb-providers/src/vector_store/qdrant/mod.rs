//! Qdrant vector search engine client.
//!
//! This module provides a complete implementation of the vector store provider interface
//! for Qdrant, supporting collection management, vector operations, and semantic search.

mod admin;
mod browser;
mod client;
mod provider;
mod registry;

pub use client::QdrantVectorStoreProvider;
