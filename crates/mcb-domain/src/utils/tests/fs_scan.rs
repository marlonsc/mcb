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
/// Filters out paths containing `/tests/` or `/test_` to focus on production source.
/// Requires `walkdir` (available in `mcb-domain`'s dev-dependencies).
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
                let lossy = path.to_string_lossy();
                if !lossy.contains("/tests/") && !lossy.contains("/test_") {
                    out.push(path);
                }
            }
        }
    }
}
