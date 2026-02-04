//! Browse-related value objects for code navigation
//!
//! These value objects support the Admin UI code browser functionality,
//! providing structured representations of indexed collections and files.

use serde::{Deserialize, Serialize};

/// Information about an indexed collection
///
/// Represents metadata about a vector store collection, including
/// statistics useful for browsing and monitoring.
///
/// # Example
///
/// ```
/// use mcb_domain::value_objects::CollectionInfo;
///
/// let info = CollectionInfo {
///     name: "my-project".to_string(),
///     vector_count: 1500,
///     file_count: 42,
///     last_indexed: Some(1705680000),
///     provider: "milvus".to_string(),
/// };
///
/// assert_eq!(info.name, "my-project");
/// assert_eq!(info.vector_count, 1500);
/// ```
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

/// Summary information about an indexed file
///
/// Provides metadata about a single file within a collection,
/// useful for file listing and navigation.
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

/// Tree node for hierarchical file navigation
///
/// Represents a node in the file tree, which can be either a directory
/// or a file. Used for tree view navigation in the Admin UI browser.
///
/// # Example
///
/// ```
/// use mcb_domain::value_objects::FileTreeNode;
///
/// let file = FileTreeNode::file(
///     "lib.rs",
///     "src/lib.rs",
///     15,
///     "rust",
/// );
/// assert!(!file.is_dir);
/// assert_eq!(file.chunk_count, Some(15));
///
/// let dir = FileTreeNode::directory("src", "src");
/// assert!(dir.is_dir);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileTreeNode {
    /// Display name of the node (file or directory name)
    pub name: String,

    /// Full path from repository root
    pub path: String,

    /// Whether this node is a directory
    pub is_dir: bool,

    /// Child nodes (empty for files)
    pub children: Vec<FileTreeNode>,

    /// Number of chunks (only for files)
    pub chunk_count: Option<u32>,

    /// Detected language (only for files)
    pub language: Option<String>,
}

impl FileTreeNode {
    /// Create a new file node
    pub fn file(
        name: impl Into<String>,
        path: impl Into<String>,
        chunk_count: u32,
        language: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: false,
            children: Vec::new(),
            chunk_count: Some(chunk_count),
            language: Some(language.into()),
        }
    }

    /// Create a new directory node
    pub fn directory(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            is_dir: true,
            children: Vec::new(),
            chunk_count: None,
            language: None,
        }
    }

    /// Add a child node to this directory
    pub fn add_child(&mut self, child: FileTreeNode) {
        self.children.push(child);
    }

    /// Sort children: directories first, then files, alphabetically
    pub fn sort_children(&mut self) {
        self.children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });
        for child in &mut self.children {
            child.sort_children();
        }
    }
}

// Tests moved to tests/unit/browse_tests.rs per test organization standards
