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
    line.starts_with("fn ")
        || line.starts_with("pub fn ")
        || line.starts_with("async fn ")
        || line.starts_with("pub async fn ")
        || line == "{"
        || line == "}"
}

// ── Block scanning ──────────────────────────────────────────────────────

/// Generic helper: iterate over lines within a brace-delimited block.
pub fn within_block<F>(lines: &[&str], start_line: usize, mut visitor: F)
where
    F: FnMut(&str, usize) -> bool,
{
    let mut brace_depth = 0;
    let mut in_block = false;

    for (idx, line) in lines[start_line..].iter().enumerate() {
        if line.contains('{') {
            in_block = true;
        }
        if in_block {
            brace_depth += line.chars().filter(|c| *c == '{').count();
            brace_depth -= line.chars().filter(|c| *c == '}').count();

            if !visitor(line, idx) {
                break;
            }

            if brace_depth == 0 {
                break;
            }
        }
    }
}

/// Count lines in a code block (impl, struct, etc.)
#[must_use]
pub fn count_block_lines(lines: &[&str], start_line: usize) -> usize {
    let mut count = 0;
    within_block(lines, start_line, |_, _| {
        count += 1;
        true
    });
    count
}

/// Generic helper: Count pattern matches within a block.
#[must_use]
pub fn count_matches_in_block(lines: &[&str], start_line: usize, pattern: &Regex) -> usize {
    let mut count = 0;
    within_block(lines, start_line, |line, _| {
        if pattern.is_match(line) {
            count += 1;
        }
        true
    });
    count
}

/// Count match arms in a match statement.
///
/// # Errors
///
/// Returns an error if the required pattern cannot be loaded.
pub fn count_match_arms(lines: &[&str], start_line: usize) -> Result<usize> {
    let arrow_pattern = required_pattern("SOLID003.match_arrow")?;

    let mut count = 0;
    let mut brace_depth = 0;

    within_block(lines, start_line, |line, _| {
        brace_depth += line.chars().filter(|c| *c == '{').count();
        brace_depth -= line.chars().filter(|c| *c == '}').count();

        if arrow_pattern.is_match(line) && brace_depth >= 1 {
            count += 1;
        }
        true
    });
    Ok(count)
}

/// Helper: Scan declaration blocks and count methods.
///
/// # Errors
///
/// Returns an error if file traversal fails.
pub fn scan_decl_blocks<F, V>(
    config: &ValidationConfig,
    decl_pattern: &Regex,
    member_fn_pattern: &Regex,
    count_fn: fn(&[&str], usize, &Regex) -> usize,
    max_allowed: usize,
    make_violation: F,
) -> Result<Vec<V>>
where
    F: Fn(PathBuf, usize, &str, usize, usize) -> V,
{
    let mut violations = Vec::new();

    for_each_rust_file(config, |path, lines| {
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(cap) = decl_pattern.captures(line) {
                let name = cap.get(1).map_or("", |m| m.as_str());
                let method_count = count_fn(&lines, line_num, member_fn_pattern);

                if method_count > max_allowed {
                    violations.push(make_violation(
                        path.clone(),
                        line_num + 1,
                        name,
                        method_count,
                        max_allowed,
                    ));
                }
            }
        }
        Ok(())
    })?;

    Ok(violations)
}

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
    let mut functions = Vec::new();

    let mut i = 0;
    while i < lines.len() {
        let (orig_idx, trimmed) = lines[i];
        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            let fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
            let fn_start = orig_idx + 1; // 1-based

            // Find function body extent by tracking braces
            let mut brace_depth: i32 = 0;
            let mut fn_started = false;
            let mut fn_end_idx = i;

            for (j, (_, line_content)) in lines[i..].iter().enumerate() {
                let opens = i32::try_from(line_content.chars().filter(|c| *c == '{').count())
                    .unwrap_or(i32::MAX);
                let closes = i32::try_from(line_content.chars().filter(|c| *c == '}').count())
                    .unwrap_or(i32::MAX);

                if opens > 0 {
                    fn_started = true;
                }
                brace_depth += opens - closes;
                if fn_started && brace_depth <= 0 {
                    fn_end_idx = i + j;
                    break;
                }
            }

            let body: Vec<String> = lines[i..=fn_end_idx]
                .iter()
                .map(|(_, l)| l.trim().to_owned())
                .filter(|l| !l.is_empty() && !l.starts_with(COMMENT_PREFIX))
                .collect();

            let meaningful = meaningful_lines(&body);
            let has_cf = has_control_flow(&body);

            functions.push(FunctionInfo {
                name: fn_name,
                start_line: fn_start,
                body_lines: body,
                meaningful_body: meaningful,
                has_control_flow: has_cf,
            });

            i = fn_end_idx;
        }
        i += 1;
    }
    functions
}

/// Extract functions with full body tracking, optionally tracking impl blocks.
pub fn extract_functions_with_body(
    fn_pattern: Option<&Regex>,
    impl_pattern: Option<&Regex>,
    lines: &[(usize, &str)],
    current_struct: &mut String,
) -> Vec<FunctionInfo> {
    let mut functions = Vec::new();
    let mut current_fn_name = String::new();
    let mut fn_start_line: usize = 0;
    let mut fn_body_lines: Vec<String> = Vec::new();
    let mut brace_depth: i32 = 0;
    let mut in_fn = false;

    for &(orig_idx, trimmed) in lines {
        if trimmed.starts_with(COMMENT_PREFIX) {
            continue;
        }

        if let Some(re) = impl_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            *current_struct = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
        }

        if let Some(re) = fn_pattern
            && let Some(cap) = re.captures(trimmed)
        {
            current_fn_name = cap
                .get(1)
                .map(|m| m.as_str().to_owned())
                .unwrap_or_default();
            fn_start_line = orig_idx + 1; // 1-based
            fn_body_lines.clear();
            in_fn = true;
            brace_depth = 0;
        }

        if in_fn {
            let opens =
                i32::try_from(trimmed.chars().filter(|c| *c == '{').count()).unwrap_or(i32::MAX);
            let closes =
                i32::try_from(trimmed.chars().filter(|c| *c == '}').count()).unwrap_or(i32::MAX);
            brace_depth += opens - closes;

            if !trimmed.is_empty() && !trimmed.starts_with("#[") {
                fn_body_lines.push(trimmed.to_owned());
            }

            if brace_depth <= 0 && opens > 0 {
                let meaningful = meaningful_lines(&fn_body_lines);
                functions.push(FunctionInfo {
                    name: current_fn_name.clone(),
                    start_line: fn_start_line,
                    body_lines: fn_body_lines.clone(),
                    meaningful_body: meaningful,
                    has_control_flow: has_control_flow(&fn_body_lines),
                });
                in_fn = false;
                fn_body_lines.clear();
            }
        }
    }
    functions
}

// ── Structural relatedness ──────────────────────────────────────────────

/// Check if structs seem related (share common prefix/suffix).
#[must_use]
pub fn structs_seem_related(names: &[String]) -> bool {
    use crate::validators::solid::constants::MIN_NAMES_FOR_RELATION_CHECK;
    if names.len() < MIN_NAMES_FOR_RELATION_CHECK {
        return true;
    }

    let checks = [
        has_common_prefix,
        has_common_suffix,
        has_purpose_suffix,
        has_shared_keyword,
        has_common_words,
    ];

    checks.iter().any(|check| check(names))
}

// ── Private helpers ─────────────────────────────────────────────────────

/// Filter a list of body lines to only meaningful ones (no braces, no `fn` sigs).
fn meaningful_lines(body: &[String]) -> Vec<String> {
    use crate::constants::common::FN_PREFIX;
    body.iter()
        .filter(|l| {
            !l.starts_with('{')
                && !l.starts_with('}')
                && *l != "{"
                && *l != "}"
                && !l.starts_with(FN_PREFIX)
        })
        .cloned()
        .collect()
}

/// Check if any line in a function body contains control-flow keywords.
fn has_control_flow(body: &[String]) -> bool {
    body.iter().any(|line| {
        line.contains(" if ")
            || line.starts_with("if ")
            || line.contains("} else")
            || line.starts_with("match ")
            || line.contains(" match ")
            || line.starts_with("for ")
            || line.starts_with("while ")
            || line.starts_with("loop ")
            || line.contains(" else {")
            || line.contains("else {")
    })
}

/// Check for common prefix (at least `MIN_AFFIX_LENGTH` chars).
fn has_common_prefix(names: &[String]) -> bool {
    use crate::validators::solid::constants::{MAX_AFFIX_LENGTH, MIN_AFFIX_LENGTH};
    let first = &names[0];
    for len in (MIN_AFFIX_LENGTH..=first.len().min(MAX_AFFIX_LENGTH)).rev() {
        let prefix = &first[..len];
        if names.iter().all(|n| n.starts_with(prefix)) {
            return true;
        }
    }
    false
}

/// Check for common suffix (at least `MIN_AFFIX_LENGTH` chars).
fn has_common_suffix(names: &[String]) -> bool {
    use crate::validators::solid::constants::{MAX_AFFIX_LENGTH, MIN_AFFIX_LENGTH};
    let first = &names[0];
    for len in (MIN_AFFIX_LENGTH..=first.len().min(MAX_AFFIX_LENGTH)).rev() {
        let suffix = &first[first.len().saturating_sub(len)..];
        if names.iter().all(|n| n.ends_with(suffix)) {
            return true;
        }
    }
    false
}

/// Check if structs share related purpose suffixes.
fn has_purpose_suffix(names: &[String]) -> bool {
    let purpose_suffixes = [
        "Config",
        "State",
        "Error",
        "Request",
        "Response",
        "Options",
        "Args",
        "Report",
        "Entry",
        "Info",
        "Data",
        "Metrics",
        "Operation",
        "Status",
        "Result",
        "Summary",
        "File",
        "Match",
        "Check",
        "Health",
        "Complexity",
    ];
    names
        .iter()
        .any(|n| purpose_suffixes.iter().any(|suffix| n.ends_with(suffix)))
}

/// Check if structs share domain keywords.
fn has_shared_keyword(names: &[String]) -> bool {
    let domain_keywords = [
        "Config",
        "Options",
        "Settings",
        "Error",
        "Result",
        "Builder",
        "Handler",
        "Provider",
        "Service",
        "Health",
        "Crypto",
        "Admin",
        "Http",
        "Args",
        "Request",
        "Response",
        "State",
        "Status",
        "Info",
        "Data",
        "Message",
        "Event",
        "Token",
        "Auth",
        "Cache",
        "Index",
        "Search",
        "Chunk",
        "Embed",
        "Vector",
        "Transport",
        "Operation",
        "Mcp",
        "Protocol",
        "Server",
        "Client",
        "Connection",
        "Session",
        "Route",
        "Endpoint",
        "Memory",
        "Observation",
        "Filter",
        "Pattern",
    ];

    domain_keywords.iter().any(|keyword| {
        let has_keyword: Vec<_> = names.iter().filter(|n| n.contains(keyword)).collect();
        has_keyword.len() > names.len() / 2
    })
}

/// Check for partial word overlaps in CamelCase names.
fn has_common_words(names: &[String]) -> bool {
    use crate::validators::solid::constants::MIN_WORD_LENGTH_FOR_COMPARISON;
    let words: Vec<Vec<&str>> = names.iter().map(|n| split_camel_case(n)).collect();

    if let Some(first_words) = words.first() {
        for word in first_words {
            if word.len() >= MIN_WORD_LENGTH_FOR_COMPARISON {
                let count = words.iter().filter(|w| w.contains(word)).count();
                if count > names.len() / 2 {
                    return true;
                }
            }
        }
    }
    false
}

/// Split a CamelCase string into words.
fn split_camel_case(s: &str) -> Vec<&str> {
    let mut words = Vec::new();
    let mut start = 0;
    for (i, c) in s.char_indices() {
        if c.is_uppercase() && i > 0 {
            if start < i {
                words.push(&s[start..i]);
            }
            start = i;
        }
    }
    if start < s.len() {
        words.push(&s[start..]);
    }
    words
}
