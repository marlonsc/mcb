//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
use std::path::Path;

use super::{QualityValidator, QualityViolation};
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};
use mcb_utils::constants::validate::{FORWARD_SEARCH_LINES, TEST_DIR_FRAGMENT};
use mcb_utils::utils::regex::compile_regex;
use regex::Regex;

/// Regexes used to attribute an `allow(dead_code)` attribute to the item it
/// annotates.
struct DeadCodePatterns {
    dead_code: Regex,
    struct_decl: Regex,
    fn_decl: Regex,
    field: Regex,
}

/// Scans for and reports usage of `allow(dead_code)` attributes.
///
/// # Errors
///
/// Returns an error if regex compilation or file scanning fails.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let patterns = DeadCodePatterns {
        dead_code: compile_regex(r"#\[allow\([^\)]*dead_code[^\)]*\)\]")?,
        struct_decl: compile_regex(r"pub\s+struct\s+(\w+)")?,
        fn_decl: compile_regex(r"(?:pub\s+)?fn\s+(\w+)")?,
        field: compile_regex(r"(?:pub\s+)?(\w+):\s+")?,
    };

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            let path = &entry.absolute_path;
            if is_skippable_file(path) {
                return Ok(());
            }
            let content = std::fs::read_to_string(path)?;
            collect_dead_code_allows(path, &content, &patterns, &mut violations);
            Ok(())
        },
    )?;

    Ok(violations)
}

/// Returns `true` for non-Rust, test, or missing files that should not be
/// scanned for `allow(dead_code)`.
fn is_skippable_file(path: &Path) -> bool {
    path.extension().is_none_or(|ext| ext != "rs")
        || path.to_str().is_some_and(|s| s.contains(TEST_DIR_FRAGMENT))
        || !path.exists()
}

/// Push a violation for every `allow(dead_code)` attribute in `content`,
/// naming the item it annotates.
fn collect_dead_code_allows(
    path: &Path,
    content: &str,
    patterns: &DeadCodePatterns,
    violations: &mut Vec<QualityViolation>,
) {
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if !patterns.dead_code.is_match(line) {
            continue;
        }
        let item_name = find_dead_code_item(
            &lines,
            i,
            &patterns.struct_decl,
            &patterns.fn_decl,
            &patterns.field,
        )
        .unwrap_or_else(|| "allow(dead_code)".to_owned());
        violations.push(QualityViolation::DeadCodeAllowNotPermitted {
            file: path.to_path_buf(),
            line: i + 1,
            item_name,
            severity: Severity::Warning,
        });
    }
}

fn find_dead_code_item(
    lines: &[&str],
    start_idx: usize,
    struct_pattern: &Regex,
    fn_pattern: &Regex,
    field_pattern: &Regex,
) -> Option<String> {
    let end = std::cmp::min(start_idx + FORWARD_SEARCH_LINES, lines.len());
    for line in lines.iter().take(end).skip(start_idx) {
        if let Some(name) = struct_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("struct {}", name.as_str()));
        }

        if let Some(name) = fn_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("fn {}", name.as_str()));
        }

        if let Some(name) = field_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("field {}", name.as_str()));
        }
    }

    None
}
