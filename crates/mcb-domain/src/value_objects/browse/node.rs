use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// File tree node structure (agnóstico, used by all renderers)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileNode {
    /// Stores the path value.
    pub path: PathBuf,
    /// Stores the name value.
    pub name: String,
    /// Stores the is dir value.
    pub is_dir: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Stores the children value.
    pub children: Option<Vec<FileNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Stores the language value.
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Stores the lines value.
    pub lines: Option<usize>,
}
