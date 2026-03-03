//!
//! **Documentation**: [docs/modules/validate.md](../../../../docs/modules/validate.md)
//!
//! Shared RCA (rust-code-analysis) helpers.
//!
//! Thin utility layer over RCA's native types — no wrappers, no duplication.
//! Provides file→FuncSpace parsing and recursive traversal with test-module detection.
//!
use std::cell::RefCell;
use std::path::Path;

use rust_code_analysis::{FuncSpace, SpaceKind, get_function_spaces};

use crate::filters::LanguageDetector;

thread_local! {
    static LANG_DETECTOR: RefCell<LanguageDetector> = RefCell::new(LanguageDetector::new());
}

/// Parse source code and return the root [`FuncSpace`].
/// Uses `ValidationRunContext` cache when available.
///
/// Returns `None` when the language cannot be detected or analysis fails.
#[must_use]
pub fn parse_file_spaces(path: &Path, content: &str) -> Option<FuncSpace> {
    // Try cached version if context is active
    if let Some(ctx) = crate::run_context::ValidationRunContext::active() {
        return ctx.parse_rca_cached(path, content);
    }
    // Fallback to uncached
    parse_file_spaces_raw(path, content)
}

/// Raw (uncached) RCA parse.
#[must_use]
pub fn parse_file_spaces_raw(path: &Path, content: &str) -> Option<FuncSpace> {
    let lang = LANG_DETECTOR.with(|d| d.borrow().detect_rca_lang(path, Some(content)))?;
    mcb_domain::trace!(
        "rca_helpers",
        "Parsing file with RCA",
        &format!(
            "file={} lang={:?} bytes={}",
            path.display(),
            lang,
            content.len()
        )
    );
    let result = get_function_spaces(&lang, content.as_bytes().to_vec(), path, None);
    if result.is_none() {
        mcb_domain::debug!(
            "rca_helpers",
            "RCA returned no spaces for file",
            &format!("file={}", path.display())
        );
    }
    result
}

/// Visit every [`FuncSpace`] recursively, calling `f` with the space and
/// whether it is inside a `#[cfg(test)]` module.
pub fn visit_spaces(root: &FuncSpace, content: &str, mut f: impl FnMut(&FuncSpace, bool)) {
    let lines: Vec<&str> = content.lines().collect();
    walk(root, &lines, false, &mut f);
}

/// Collect all spaces matching a given [`SpaceKind`], skipping test modules.
#[must_use]
pub fn collect_spaces_of_kind<'a>(
    root: &'a FuncSpace,
    content: &'a str,
    kind: SpaceKind,
) -> Vec<&'a FuncSpace> {
    let mut results = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    collect_walk(root, &lines, false, kind, &mut results);
    results
}

// ── private ──────────────────────────────────────────────────────────

fn walk(
    space: &FuncSpace,
    lines: &[&str],
    parent_is_test: bool,
    f: &mut impl FnMut(&FuncSpace, bool),
) {
    let is_test = parent_is_test || is_test_module(space, lines);
    f(space, is_test);
    for child in &space.spaces {
        walk(child, lines, is_test, f);
    }
}

fn collect_walk<'a>(
    space: &'a FuncSpace,
    lines: &[&str],
    parent_is_test: bool,
    kind: SpaceKind,
    results: &mut Vec<&'a FuncSpace>,
) {
    let is_test = parent_is_test || is_test_module(space, lines);
    if !is_test && space.kind == kind {
        results.push(space);
    }
    for child in &space.spaces {
        collect_walk(child, lines, is_test, kind, results);
    }
}

fn is_test_module(space: &FuncSpace, lines: &[&str]) -> bool {
    if space.kind == SpaceKind::Function {
        return false;
    }
    let start_idx = space.start_line.saturating_sub(1);
    let search_start = start_idx.saturating_sub(3);
    lines[search_start..=start_idx.min(lines.len().saturating_sub(1))]
        .iter()
        .any(|l| l.contains("#[cfg(test)]"))
}
