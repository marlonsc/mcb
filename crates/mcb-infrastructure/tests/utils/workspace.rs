use std::path::{Path, PathBuf};

#[allow(clippy::missing_errors_doc)]
pub fn workspace_root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for dir in manifest_dir.ancestors() {
        if dir.join("Cargo.lock").exists() {
            return Ok(dir.to_path_buf());
        }
    }
    Err("workspace root not found from CARGO_MANIFEST_DIR ancestors".into())
}

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
