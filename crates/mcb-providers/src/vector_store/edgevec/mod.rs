//! `EdgeVec` Vector Store Provider
//!
//! High-performance embedded vector database implementation using `EdgeVec`.
//! `EdgeVec` provides sub-millisecond vector similarity search with HNSW algorithm.
//! This implementation uses the Actor pattern to eliminate locks and ensure non-blocking operation.

mod actor;
mod client;
/// `EdgeVec` provider configuration types.
pub mod config;
mod provider;
mod registry;

pub use client::EdgeVecVectorStoreProvider;
pub use config::{EdgeVecConfig, HnswConfig, MetricType, QuantizerConfig};

// Re-export internal types for sibling modules that use `super::*`
use std::collections::HashMap;

use async_trait::async_trait;
use client::*;
use dashmap::DashMap;
use edgevec::hnsw::VectorId;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{VectorStoreAdmin, VectorStoreBrowser, VectorStoreProvider};
use mcb_domain::value_objects::{CollectionId, CollectionInfo, Embedding, FileInfo, SearchResult};
use mcb_utils::utils::id;
use tokio::sync::mpsc;
