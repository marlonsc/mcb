//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
use std::path::{Path, PathBuf};

/// Explorer for workspace structure and crate discovery.
///
/// Centralizes logic for locating the workspace root and identifying
/// crate source directories, ensuring a Single Source of Truth for
/// workspace structure navigation.
pub struct WorkspaceExplorer;

impl WorkspaceExplorer {
    /// Find workspace root starting from a given path.
    ///
    /// Traverses up the directory tree looking for a `Cargo.toml` file
    /// that contains a `[workspace]` section.
    #[must_use]
    pub fn find_workspace_root_from(start: &Path) -> Option<PathBuf> {
        let mut current = start.to_path_buf();
        loop {
            let cargo_toml = current.join("Cargo.toml");
            if cargo_toml.exists()
                && std::fs::read_to_string(&cargo_toml)
                    .is_ok_and(|content| content.contains("[workspace]"))
            {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Find workspace root from current directory.
    #[must_use]
    pub fn find_workspace_root() -> Option<PathBuf> {
        let current = std::env::current_dir().ok()?;
        Self::find_workspace_root_from(&current)
    }

    /// Scan for crate source directories in standard locations.
    ///
    /// Primarily looks in `crates/<crate_name>/src`.
    /// Returns paths to `src/` directories of detected crates.
    #[must_use]
    pub fn scan_crate_sources(workspace_root: &Path, exclude_patterns: &[&str]) -> Vec<PathBuf> {
        let crates_dir = workspace_root.join("crates");

        std::fs::read_dir(crates_dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|entry| Self::crate_src_dir(&entry.path(), exclude_patterns))
            .collect()
    }

    /// Return the `src/` directory for a crate path when it is a non-excluded
    /// directory that contains a `src/` folder.
    fn crate_src_dir(path: &Path, exclude_patterns: &[&str]) -> Option<PathBuf> {
        if !path.is_dir() {
            return None;
        }
        let path_str = path.to_string_lossy();
        if exclude_patterns.iter().any(|p| path_str.contains(*p)) {
            return None;
        }
        let src = path.join("src");
        src.exists().then_some(src)
    }
}
