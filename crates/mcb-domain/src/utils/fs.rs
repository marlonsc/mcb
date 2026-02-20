//!
//! **Documentation**: [docs/modules/domain.md](../../../../docs/modules/domain.md)
//!
//! Generic filesystem utilities for the MCB workspace.

use crate::error::{Error, Result};
use std::path::{Path, PathBuf};

fn read_entries(dir: &Path) -> Result<std::fs::ReadDir> {
    std::fs::read_dir(dir)
        .map_err(|e| Error::internal(format!("Failed to read directory {}: {}", dir.display(), e)))
}

fn read_file_type(entry: &std::fs::DirEntry, path: &Path) -> Result<std::fs::FileType> {
    entry.file_type().map_err(|e| {
        Error::internal(format!(
            "Failed to get file type for {}: {}",
            path.display(),
            e
        ))
    })
}

fn has_matching_extension(path: &Path, extensions: &[&str]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| extensions.iter().any(|&e| e.eq_ignore_ascii_case(ext)))
}

/// Collect all files recursively from a directory that match specific extensions.
///
/// # Errors
///
/// Returns an error if directory traversal fails.
pub fn find_files_by_extensions(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = read_entries(&dir)?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::internal(format!(
                    "Failed to read directory entry in {}: {}",
                    dir.display(),
                    e
                ))
            })?;
            let path = entry.path();
            let file_type = read_file_type(&entry, &path)?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file() && has_matching_extension(&path, extensions) {
                files.push(path);
            }
        }
    }

    Ok(files)
}
