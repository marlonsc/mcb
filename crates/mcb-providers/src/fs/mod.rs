use std::path::{Path, PathBuf};

use async_trait::async_trait;
use mcb_domain::error::Error;
use mcb_domain::ports::{DirEntry, FileSystemProvider};
use mcb_domain::registry::fs::{
    FILE_SYSTEM_PROVIDERS, FileSystemProviderConfig, FileSystemProviderEntry,
};

fn create_local_file_system_provider(
    _config: &FileSystemProviderConfig,
) -> std::result::Result<std::sync::Arc<dyn FileSystemProvider>, String> {
    Ok(std::sync::Arc::new(LocalFileSystemProvider::new()))
}

#[linkme::distributed_slice(FILE_SYSTEM_PROVIDERS)]
static LOCAL_FILE_SYSTEM_PROVIDER: FileSystemProviderEntry = FileSystemProviderEntry {
    name: "local",
    description: "Local file-system provider",
    build: create_local_file_system_provider,
};

#[allow(missing_docs)]
#[derive(Debug, Default)]
pub struct LocalFileSystemProvider;

#[allow(missing_docs)]
impl LocalFileSystemProvider {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn read_dir_sync(path: &Path) -> Result<Vec<DirEntry>, Error> {
        let entries = std::fs::read_dir(path).map_err(|e| {
            Error::internal(format!("Failed to read directory {}: {e}", path.display()))
        })?;

        let mut out = Vec::new();
        for entry in entries {
            let entry = entry
                .map_err(|e| Error::internal(format!("Failed to read directory entry: {e}")))?;
            let file_type = entry
                .file_type()
                .map_err(|e| Error::internal(format!("Failed to read file type: {e}")))?;

            out.push(DirEntry {
                path: entry.path(),
                is_file: file_type.is_file(),
                is_dir: file_type.is_dir(),
            });
        }

        Ok(out)
    }
}

#[async_trait]
impl FileSystemProvider for LocalFileSystemProvider {
    async fn read_to_string(&self, path: &Path) -> Result<String, Error> {
        std::fs::read_to_string(path)
            .map_err(|e| Error::internal(format!("Failed to read file {}: {e}", path.display())))
    }

    async fn read_dir_entries(&self, path: &Path) -> Result<Vec<DirEntry>, Error> {
        Self::read_dir_sync(path)
    }

    async fn canonicalize_path(&self, path: &Path) -> Result<PathBuf, Error> {
        std::fs::canonicalize(path)
            .map_err(|e| Error::internal(format!("Failed to canonicalize {}: {e}", path.display())))
    }
}
