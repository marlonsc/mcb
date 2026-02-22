use std::path::{Path, PathBuf};

use async_trait::async_trait;

use crate::error::Error;

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirEntry {
    pub path: PathBuf,
    pub is_file: bool,
    pub is_dir: bool,
}

#[allow(missing_docs)]
#[async_trait]
pub trait FileSystemProvider: Send + Sync {
    async fn read_to_string(&self, path: &Path) -> std::result::Result<String, Error>;

    async fn read_dir_entries(&self, path: &Path) -> std::result::Result<Vec<DirEntry>, Error>;

    async fn canonicalize_path(&self, path: &Path) -> std::result::Result<PathBuf, Error>;
}
