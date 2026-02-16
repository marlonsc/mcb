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

    for (line_num, line) in lines.iter().enumerate() {
        // Find trait definitions
        if let Some(cap) = trait_pattern.captures(line) {
            let trait_name = cap.get(1).map_or("", |m| m.as_str());

            // Look ahead to see if trait has async methods
            let mut has_async_methods = false;

            // Use shared scan helper
            if let Some((body_lines, _)) = crate::scan::extract_balanced_block(&lines, line_num) {
                for subsequent_line in body_lines {
                    if async_fn_pattern.is_match(subsequent_line) {
                        has_async_methods = true;
                        break;
                    }
                }
            }

            if has_async_methods {
                let has_async_trait_attr = if line_num > 0 {
                    lines[..line_num]
                        .iter()
                        .rev()
                        .take(ATTR_SEARCH_LINES)
                        .any(|l| async_trait_attr.is_match(l) || allow_async_fn_trait.is_match(l))
                } else {
                    false
                };

                // Check if using native async trait support
                let uses_native_async = if line_num > 0 {
                    lines[..line_num]
                        .iter()
                        .rev()
                        .take(ATTR_SEARCH_LINES)
                        .any(|l| allow_async_fn_trait.is_match(l))
                } else {
                    false
                };

                if !has_async_trait_attr {
                    violations.push(PatternViolation::MissingAsyncTrait {
                        file: path.to_path_buf(),
                        line: line_num + 1,
                        trait_name: trait_name.to_owned(),
                        severity: Severity::Error,
                    });
                }

                // Check for Send + Sync bounds (skip for native async traits)
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
    }
    Ok(violations)
}
