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
        let mut scan_dirs = Vec::new();
        let crates_dir = workspace_root.join("crates");

        if crates_dir.exists() {
            for entry in std::fs::read_dir(crates_dir)
                .into_iter()
                .flatten()
                .flatten()
            {
                let path = entry.path();
                if path.is_dir() {
                    // Check excludes
                    let path_str = path.to_string_lossy();
                    if exclude_patterns.iter().any(|p| path_str.contains(*p)) {
                        continue;
                    }

                    let src = path.join("src");
                    if src.exists() {
                        scan_dirs.push(src);
                    }
                }
            }
        }
        scan_dirs
    }
}
