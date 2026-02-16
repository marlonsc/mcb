//! Source code analysis utilities.
//!
//! Provides function extraction, brace-delimited block tracking, declaration
//! scanning, and structural relatedness checks for Rust source files.

use std::path::PathBuf;

use regex::Regex;

use crate::constants::common::{CFG_TEST_MARKER, COMMENT_PREFIX};
use crate::filters::LanguageId;
use crate::pattern_registry::required_pattern;
use crate::scan::for_each_file_under_root;
use crate::{Result, ValidationConfig};

#[path = "source_blocks.rs"]
mod source_blocks;
#[path = "source_functions.rs"]
mod source_functions;
#[path = "source_relatedness.rs"]
mod source_relatedness;

pub use source_blocks::{
    DeclScanConfig, count_block_lines, count_match_arms, count_matches_in_block, scan_decl_blocks,
    within_block,
};

// ── File traversal ──────────────────────────────────────────────────────

/// Generic helper: iterate over all Rust files in crate source directories.
///
/// # Errors
///
/// Returns an error if directory enumeration or file reading fails.
pub fn for_each_rust_file<F>(config: &ValidationConfig, mut visitor: F) -> Result<()>
where
    F: FnMut(PathBuf, Vec<&str>) -> Result<()>,
{
    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        for_each_file_under_root(config, &src_dir, Some(LanguageId::Rust), |entry| {
            let path = &entry.absolute_path;
            let content = std::fs::read_to_string(path)?;
            let lines: Vec<&str> = content.lines().collect();
            visitor(path.clone(), lines)
        })?;
    }
    Ok(())
}

// ── Line filtering ──────────────────────────────────────────────────────

/// Iterate source lines, skipping comments and `#[cfg(test)]` modules.
/// Yields `(1-based line number, trimmed line)`.
#[must_use]
pub fn source_lines(content: &str) -> Vec<(usize, &str)> {
    let mut result = Vec::new();
    let mut in_test_module = false;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }
        if trimmed.contains(CFG_TEST_MARKER) {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        result.push((idx + 1, trimmed));
    }
    result
}

/// Filter out lines that belong to `#[cfg(test)]` regions.
/// Returns `(original 0-based index, trimmed line)` pairs.
#[must_use]
pub fn non_test_lines<'a>(lines: &[&'a str]) -> Vec<(usize, &'a str)> {
    let mut result = Vec::new();
    let mut in_test_module = false;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains(CFG_TEST_MARKER) {
            in_test_module = true;
            continue;
        }
        if in_test_module {
            continue;
        }
        result.push((i, trimmed));
    }
    result
}

// ── Pattern helpers ─────────────────────────────────────────────────────

/// Track function name from a regex pattern match.
pub fn track_fn_name(fn_pattern: Option<&Regex>, trimmed: &str, name: &mut String) {
    if let Some(re) = fn_pattern
        && let Some(cap) = re.captures(trimmed)
    {
        *name = cap
            .get(1)
            .map(|m| m.as_str().to_owned())
            .unwrap_or_default();
    }
}

/// Compile `(pattern_id, description)` pairs into `(Regex, &str)`.
///
/// # Errors
///
/// Returns an error if any requested pattern ID is missing.
pub fn compile_pattern_pairs<'a>(
    ids: &[(&str, &'a str)],
) -> Result<Vec<(&'static Regex, &'a str)>> {
    ids.iter()
        .map(|(id, desc)| required_pattern(id).map(|r| (r, *desc)))
        .collect()
}

/// Compile an iterator of pattern IDs into a `Vec<&Regex>`.
///
/// # Errors
///
/// Returns an error if any requested pattern ID is missing.
pub fn required_patterns<'a>(ids: impl Iterator<Item = &'a str>) -> Result<Vec<&'static Regex>> {
    ids.map(required_pattern).collect()
}

/// Check if a line is a function signature or standalone brace.
#[must_use]
pub fn is_fn_signature_or_brace(line: &str) -> bool {
    const FN_PREFIXES: [&str; 4] = ["fn ", "pub fn ", "async fn ", "pub async fn "];
    matches!(line, "{" | "}") || FN_PREFIXES.iter().any(|prefix| line.starts_with(prefix))
}

// ── Block scanning ──────────────────────────────────────────────────────

// ── Function extraction ─────────────────────────────────────────────────

/// A parsed function with its body and metadata.
pub struct FunctionInfo {
    /// Function name.
    pub name: String,
    /// 1-based start line in the source file.
    pub start_line: usize,
    /// All non-empty, non-comment body lines.
    pub body_lines: Vec<String>,
    /// Body lines excluding braces and `fn` signatures.
    pub meaningful_body: Vec<String>,
    /// Whether the body contains control-flow keywords.
    pub has_control_flow: bool,
}

/// Extract function bodies from non-test source lines.
/// Returns structured function info for each detected function.
#[must_use]
pub fn extract_functions(fn_pattern: Option<&Regex>, lines: &[(usize, &str)]) -> Vec<FunctionInfo> {
    source_functions::extract_functions_impl(fn_pattern, lines)
}

/// Extract functions with full body tracking, optionally tracking impl blocks.
pub fn extract_functions_with_body(
    fn_pattern: Option<&Regex>,
    impl_pattern: Option<&Regex>,
    lines: &[(usize, &str)],
    current_struct: &mut String,
) -> Vec<FunctionInfo> {
    source_functions::extract_functions_with_body_impl(
        fn_pattern,
        impl_pattern,
        lines,
        current_struct,
    )
}

// ── Structural relatedness ──────────────────────────────────────────────

/// Check if structs seem related (share common prefix/suffix).
#[must_use]
pub fn structs_seem_related(names: &[String]) -> bool {
    source_relatedness::structs_seem_related_impl(names)
}

// ── Private helpers ─────────────────────────────────────────────────────
