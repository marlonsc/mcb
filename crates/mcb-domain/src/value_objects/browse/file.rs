use serde::{Deserialize, Serialize};

/// Summary information about an indexed file
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInfo {
    /// Relative path of the file within the indexed codebase
    pub path: String,
    /// Number of code chunks extracted from this file
    pub chunk_count: u32,
    /// Detected programming language
    pub language: String,
    /// File size in bytes (if available)
    pub size_bytes: Option<u64>,
}

impl FileInfo {
    /// Create a new FileInfo instance
    pub fn new(
        path: impl Into<String>,
        chunk_count: u32,
        language: impl Into<String>,
        size_bytes: Option<u64>,
    ) -> Self {
        Self {
            path: path.into(),
            chunk_count,
            language: language.into(),
            size_bytes,
        }
    }
}
