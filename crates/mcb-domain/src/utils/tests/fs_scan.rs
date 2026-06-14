//! Filesystem scanning helpers for tests.
//!
//! Centralized in `mcb-domain` to provide a single implementation of
//! `scan_rs_files` and `rust_files_under` used by DI enforcement and
//! architecture tests across multiple crates.

use std::fs;
use std::path::{Path, PathBuf};

/// Recursively collect `.rs` files under `path` (push into `out`).
///
/// Simple variant without any filtering — collects ALL Rust files.
pub fn rust_files_under(path: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            rust_files_under(&p, out);
        } else if is_rs_file(&p) {
            out.push(p);
        }
    }
}

/// Returns true when the path has a `.rs` extension.
fn is_rs_file(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()) == Some("rs")
}

/// Returns true when the directory name marks a test directory.
fn is_test_dir_name(name: &str) -> bool {
    name == "tests" || name.starts_with("test_")
}

/// Returns true when any path component is a test directory.
fn path_in_test_dir(path: &Path) -> bool {
    path.components().any(|c| {
        if let std::path::Component::Normal(os) = c {
            is_test_dir_name(&os.to_string_lossy())
        } else {
            false
        }
    })
}

/// Collect `.rs` source files under `dir`, skipping test directories.
///
/// Filters out paths containing test directories (`tests` or `test_*` components)
/// to focus on production source. Uses `Path` component iteration for correct
/// cross-platform behaviour (including Windows paths with `\\`).
#[must_use]
pub fn scan_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if !dir.exists() {
        return results;
    }
    collect_rs_files_filtered(dir, &mut results);
    results
}

fn collect_rs_files_filtered(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !is_test_dir_name(name) {
                collect_rs_files_filtered(&path, out);
            }
        } else if is_rs_file(&path) && !path_in_test_dir(&path) {
            out.push(path);
        }
    }
}
