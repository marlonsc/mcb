//! Internal types for filesystem vector store
//!
//! Contains shard metadata and index entry structures used internally
//! by the filesystem vector store implementation.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Vector shard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct ShardMetadata {
    /// Shard ID
    pub shard_id: u32,
    /// Number of vectors in this shard
    pub vector_count: usize,
    /// File offset for the start of vectors
    pub vectors_offset: u64,
    /// File size for vectors section
    pub vectors_size: u64,
    /// Creation timestamp
    pub created_at: u64,
}

/// Vector index entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct IndexEntry {
    /// Vector ID
    pub id: String,
    /// Shard ID where vector is stored
    pub shard_id: u32,
    /// Offset within the shard file
    pub offset: u64,
    /// Vector metadata
    pub metadata: HashMap<String, serde_json::Value>,
}
