//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Filesystem utilities for file collection and traversal.

use crate::Result;
use std::path::{Path, PathBuf};

/// Collect all YAML files recursively from a directory.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>> {
    mcb_utils::utils::fs::find_files_by_extensions(root, &["yml", "yaml"])
        .map_err(|e| crate::ValidationError::Config(e.to_string()))
}
