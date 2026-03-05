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
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                rust_files_under(&p, out);
            } else if p.extension().and_then(|e| e.to_str()) == Some("rs") {
                out.push(p);
            }
        }
    }
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
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                // Skip test directories
                if name == "tests" || name.starts_with("test_") {
                    continue;
                }
                collect_rs_files_filtered(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                // Double-check via Path components for correct cross-platform behaviour.
                let in_test_dir = path.components().any(|c| {
                    if let std::path::Component::Normal(os) = c {
                        let s = os.to_string_lossy();
                        s == "tests" || s.starts_with("test_")
                    } else {
                        false
                    }
                });
                if !in_test_dir {
                    out.push(path);
                }
            }
        }
    }
}
