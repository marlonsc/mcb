//! File discovery and filtering for indexing.
//!
//! This module handles recursive directory traversal, file filtering,
//! and extension validation during the indexing process.

use std::path::Path;

use ignore::WalkBuilder;

use crate::constants::use_cases::SKIP_DIRS;

impl IndexingServiceImpl {
    /// Discover files recursively from a path
    pub(crate) async fn discover_files(
        &self,
        path: &Path,
        progress: &mut IndexingProgress,
    ) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .filter_entry(|entry| {
                if !entry.file_type().is_some_and(|ft| ft.is_dir()) {
                    return true;
                }

                entry
                    .file_name()
                    .to_str()
                    .is_none_or(|name| !SKIP_DIRS.contains(&name))
            })
            .build();

        for entry_result in walker {
            match entry_result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|ft| ft.is_file())
                        && self.is_supported_file(entry.path())
                    {
                        files.push(entry.path().to_path_buf());
                    }
                }
                Err(e) => {
                    progress.record_error("Failed to read directory entry", path, e);
                }
            }
        }

        files
    }

    /// Check if file has a supported extension
    fn is_supported_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| {
                let ext_lower = ext.to_ascii_lowercase();
                self.supported_extensions.contains(&ext_lower)
            })
    }
}
