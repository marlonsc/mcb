//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::violation::PatternViolation;
use crate::constants::common::ATTR_SEARCH_LINES;
use crate::pattern_registry::compile_regex;
use crate::traits::violation::Severity;

/// Checks for async trait usage in a single file.
pub fn check_async_traits(path: &Path, content: &str) -> crate::Result<Vec<PatternViolation>> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let trait_pattern = compile_regex(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)")?;
    let async_fn_pattern = compile_regex(r"async\s+fn\s+")?;
    let send_sync_pattern = compile_regex(r":\s*.*Send\s*\+\s*Sync")?;
    let async_trait_attr = compile_regex(r"#\[(async_trait::)?async_trait\]")?;
    let allow_async_fn_trait = compile_regex(r"#\[allow\(async_fn_in_trait\)\]")?;

    let has_async_methods = |trait_line: usize| {
        crate::scan::extract_balanced_block(&lines, trait_line).is_some_and(|(body_lines, _)| {
            body_lines.into_iter().any(|l| async_fn_pattern.is_match(l))
        })
    };

    let has_attr_nearby = |line_num: usize, pattern: &regex::Regex| {
        line_num > 0
            && lines[..line_num]
                .iter()
                .rev()
                .take(ATTR_SEARCH_LINES)
                .any(|l| pattern.is_match(l))
    };

    for (line_num, line) in lines.iter().enumerate() {
        if let Some(cap) = trait_pattern.captures(line) {
            let trait_name = cap.get(1).map_or("", |m| m.as_str());
            if !has_async_methods(line_num) {
                continue;
            }

            let uses_native_async = has_attr_nearby(line_num, &allow_async_fn_trait);
            let has_async_trait_attr =
                uses_native_async || has_attr_nearby(line_num, &async_trait_attr);

            if !has_async_trait_attr {
                violations.push(PatternViolation::MissingAsyncTrait {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    trait_name: trait_name.to_owned(),
                    severity: Severity::Error,
                });
            }

            if !send_sync_pattern.is_match(line) && !uses_native_async {
                violations.push(PatternViolation::MissingSendSync {
                    file: path.to_path_buf(),
                    line: line_num + 1,
                    trait_name: trait_name.to_owned(),
                    missing_bound: "Send + Sync".to_owned(),
                    severity: Severity::Warning,
                });
            }
        }
    }
    Ok(violations)
}
