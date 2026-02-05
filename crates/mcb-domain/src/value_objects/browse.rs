//! Browse-related value objects for code navigation
//!
//! These value objects support the Admin UI code browser functionality,
//! providing structured representations of indexed collections and files.

use serde::{Deserialize, Serialize};
use std::fmt;

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

    /// Traverse the tree and call a callback for each node
    ///
    /// Performs a depth-first traversal of the tree, calling the provided callback
    /// for each node (including the root).
    ///
    /// # Arguments
    ///
    /// * `callback` - Function to call for each node
    ///
    /// # Example
    ///
    /// ```
    /// use mcb_domain::value_objects::FileTreeNode;
    ///
    /// let mut root = FileTreeNode::directory("src", "src");
    /// root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));
    ///
    /// let mut count = 0;
    /// root.traverse(&mut |_node| {
    ///     count += 1;
    /// });
    /// assert_eq!(count, 2); // root + 1 child
    /// ```
    pub fn traverse(&self, callback: &mut dyn FnMut(&FileTreeNode)) {
        callback(self);
        for child in &self.children {
            child.traverse(callback);
        }
    }

    /// Convert tree to ANSI-formatted string with colors and tree structure
    ///
    /// Returns a string representation of the tree with ANSI color codes for
    /// terminal display. Directories are shown in blue, files in default color.
    ///
    /// # Example
    ///
    /// ```
    /// use mcb_domain::value_objects::FileTreeNode;
    ///
    /// let mut root = FileTreeNode::directory("src", "src");
    /// root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));
    ///
    /// let ansi = root.to_ansi();
    /// assert!(ansi.contains("src"));
    /// assert!(ansi.contains("lib.rs"));
    /// ```
    pub fn to_ansi(&self) -> String {
        let mut output = String::new();
        self.format_ansi(&mut output, "", true);
        output
    }

    fn format_ansi(&self, output: &mut String, prefix: &str, is_last: bool) {
        let connector = if is_last { "└── " } else { "├── " };
        let color = if self.is_dir { "\x1b[34m" } else { "\x1b[0m" };
        let reset = "\x1b[0m";

        output.push_str(prefix);
        output.push_str(connector);
        output.push_str(color);
        output.push_str(&self.name);
        output.push_str(reset);

        if let Some(count) = self.chunk_count {
            output.push_str(&format!(" ({})", count));
        }
        output.push('\n');

        let new_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        for (i, child) in self.children.iter().enumerate() {
            let is_last_child = i == self.children.len() - 1;
            child.format_ansi(output, &new_prefix, is_last_child);
        }
    }

    /// Convert tree to HTML-formatted string with nesting
    ///
    /// Returns an HTML representation of the tree using nested `<ul>` and `<li>` elements.
    /// Directories are marked with a folder icon, files with a file icon.
    ///
    /// # Example
    ///
    /// ```
    /// use mcb_domain::value_objects::FileTreeNode;
    ///
    /// let mut root = FileTreeNode::directory("src", "src");
    /// root.add_child(FileTreeNode::file("lib.rs", "src/lib.rs", 10, "rust"));
    ///
    /// let html = root.to_html();
    /// assert!(html.contains("<ul>"));
    /// assert!(html.contains("src"));
    /// assert!(html.contains("lib.rs"));
    /// ```
    pub fn to_html(&self) -> String {
        let mut output = String::new();
        self.format_html(&mut output);
        output
    }

    fn format_html(&self, output: &mut String) {
        let icon = if self.is_dir { "📁" } else { "📄" };
        let name_html = html_escape(&self.name);

        output.push_str("<ul>\n");
        output.push_str("<li>");
        output.push_str(icon);
        output.push(' ');
        output.push_str(&name_html);

        if let Some(count) = self.chunk_count {
            output.push_str(&format!(" <span style=\"color: #888;\">({})</span>", count));
        }

        if !self.children.is_empty() {
            output.push('\n');
            for child in &self.children {
                child.format_html(output);
            }
            output.push_str("</li>\n");
            output.push_str("</ul>\n");
        } else {
            output.push_str("</li>\n");
            output.push_str("</ul>\n");
        }
    }
}

/// HTML escape a string to prevent XSS
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

impl fmt::Display for FileTreeNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_ansi())
    }
}

// Tests moved to tests/unit/browse_tests.rs per test organization standards
