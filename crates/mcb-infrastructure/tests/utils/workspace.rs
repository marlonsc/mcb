use std::path::{Path, PathBuf};

pub use mcb_domain::test_utils::workspace_root;

pub fn scan_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }
    for entry in walkdir::WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && !path.to_string_lossy().contains("/tests/")
            && !path.to_string_lossy().contains("/test_")
        {
            results.push(path.to_path_buf());
        }
    }
    results
}
