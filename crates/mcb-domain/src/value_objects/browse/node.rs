use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// File tree node structure (agnóstico, used by all renderers)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileNode {
    /// Stores the path value.
    pub path: PathBuf,
    /// Stores the name value.
    pub name: String,
    /// Stores the is dir value.
    pub is_dir: bool,
    /// Stores the children value.
    pub children: Option<Vec<FileNode>>,
    /// Stores the language value.
    pub language: Option<String>,
    /// Stores the lines value.
    pub lines: Option<usize>,
}
