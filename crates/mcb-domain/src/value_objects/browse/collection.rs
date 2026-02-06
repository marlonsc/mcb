use serde::{Deserialize, Serialize};

/// Information about an indexed collection
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionInfo {
    /// Name of the collection
    pub name: String,
    /// Total number of vectors in the collection
    pub vector_count: u64,
    /// Number of unique files indexed in the collection
    pub file_count: u64,
    /// Unix timestamp of last indexing operation (if available)
    pub last_indexed: Option<u64>,
    /// Name of the vector store provider (e.g., "milvus", "in_memory")
    pub provider: String,
}

impl CollectionInfo {
    /// Create a new CollectionInfo instance
    pub fn new(
        name: impl Into<String>,
        vector_count: u64,
        file_count: u64,
        last_indexed: Option<u64>,
        provider: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            vector_count,
            file_count,
            last_indexed,
            provider: provider.into(),
        }
    }
}
