use serde::{Deserialize, Serialize};

use crate::value_objects::CollectionId;

/// Information about an indexed collection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// ID of the collection
    pub id: CollectionId,
    /// Total number of vectors in the collection
    pub vector_count: u64,
    /// Number of unique files indexed in the collection
    pub file_count: u64,
    /// Unix timestamp of last indexing operation (if available)
    pub last_indexed: Option<u64>,
    /// Name of the vector store provider (e.g., "milvus", "qdrant")
    pub provider: String,
}

impl CollectionInfo {
    /// Create a new CollectionInfo instance
    pub fn new(
        id: impl Into<CollectionId>,
        vector_count: u64,
        file_count: u64,
        last_indexed: Option<u64>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            vector_count,
            file_count,
            last_indexed,
            provider: provider.into(),
        }
    }
}
