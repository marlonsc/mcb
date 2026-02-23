//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Filesystem utilities re-exported from `mcb-domain`.

use crate::Result;
use std::path::{Path, PathBuf};

/// Collect all YAML files recursively from a directory.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>> {
    mcb_domain::utils::fs::find_files_by_extensions(root, &["yml", "yaml"])
        .map_err(|e| crate::ValidationError::Config(e.to_string()))
}
