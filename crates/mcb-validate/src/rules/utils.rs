use std::path::{Path, PathBuf};

use crate::Result;

/// Collect all YAML files recursively from a directory
pub fn collect_yaml_files(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).map_err(crate::ValidationError::Io)? {
            let entry = entry.map_err(crate::ValidationError::Io)?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(crate::ValidationError::Io)?;

            if file_type.is_dir() {
                stack.push(path);
                continue;
            }

            if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yml") {
                files.push(path);
            }
        }
    }

    Ok(files)
}
