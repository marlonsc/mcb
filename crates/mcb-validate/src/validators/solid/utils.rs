use crate::filters::LanguageId;
use crate::pattern_registry::required_pattern;
use crate::scan::for_each_file_under_root;
use crate::{Result, ValidationConfig};
use regex::Regex;
use std::path::PathBuf;

/// Generic helper: iterate over all Rust files in crate source directories
///
/// # Errors
///
/// Returns an error if directory enumeration or file reading fails.
pub fn for_each_rust_file<F>(config: &ValidationConfig, mut visitor: F) -> Result<()>
where
    F: FnMut(PathBuf, Vec<&str>) -> Result<()>,
{
    // Use get_source_dirs from config
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

/// Helper: Scan declaration blocks and count methods
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

/// Generic helper: iterate over lines within a brace-delimited block
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

/// Generic helper: Count pattern matches within a block
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

/// Count match arms in a match statement
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

/// Check if structs seem related (share common prefix/suffix).
#[must_use]
pub fn structs_seem_related(names: &[String]) -> bool {
    if names.len() < 2 {
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

/// Check for common prefix (at least 3 chars)
fn has_common_prefix(names: &[String]) -> bool {
    let first = &names[0];
    for len in (3..=first.len().min(10)).rev() {
        let prefix = &first[..len];
        if names.iter().all(|n| n.starts_with(prefix)) {
            return true;
        }
    }
    false
}

/// Check for common suffix (at least 3 chars)
fn has_common_suffix(names: &[String]) -> bool {
    let first = &names[0];
    for len in (3..=first.len().min(10)).rev() {
        let suffix = &first[first.len().saturating_sub(len)..];
        if names.iter().all(|n| n.ends_with(suffix)) {
            return true;
        }
    }
    false
}

/// Check if structs share related purpose suffixes
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

/// Check if structs share domain keywords
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
    let words: Vec<Vec<&str>> = names.iter().map(|n| split_camel_case(n)).collect();

    if let Some(first_words) = words.first() {
        for word in first_words {
            if word.len() >= 4 {
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
