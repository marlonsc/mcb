use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use super::violation::PatternViolation;
use crate::traits::violation::Severity;

static TRAIT_PATTERN: OnceLock<Regex> = OnceLock::new();
static ASYNC_FN_PATTERN: OnceLock<Regex> = OnceLock::new();
static SEND_SYNC_PATTERN: OnceLock<Regex> = OnceLock::new();
static ASYNC_TRAIT_ATTR: OnceLock<Regex> = OnceLock::new();
static ALLOW_ASYNC_FN_TRAIT: OnceLock<Regex> = OnceLock::new();

/// Checks for async trait usage in a single file.
pub fn check_async_traits(path: &Path, content: &str) -> Vec<PatternViolation> {
    let mut violations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let trait_pattern = TRAIT_PATTERN.get_or_init(|| {
        Regex::new(r"pub\s+trait\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid trait pattern")
    });
    let async_fn_pattern = ASYNC_FN_PATTERN
        .get_or_init(|| Regex::new(r"async\s+fn\s+").expect("Invalid async fn pattern"));
    let send_sync_pattern = SEND_SYNC_PATTERN
        .get_or_init(|| Regex::new(r":\s*.*Send\s*\+\s*Sync").expect("Invalid send sync pattern"));
    let async_trait_attr = ASYNC_TRAIT_ATTR.get_or_init(|| {
        Regex::new(r"#\[(async_trait::)?async_trait\]").expect("Invalid async trait attr pattern")
    });
    let allow_async_fn_trait = ALLOW_ASYNC_FN_TRAIT.get_or_init(|| {
        Regex::new(r"#\[allow\(async_fn_in_trait\)\]")
            .expect("Invalid allow async fn trait pattern")
    });

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
                let has_async_trait_attr =
                    if line_num > 0 {
                        lines[..line_num].iter().rev().take(5).any(|l| {
                            async_trait_attr.is_match(l) || allow_async_fn_trait.is_match(l)
                        })
                    } else {
                        false
                    };

                // Check if using native async trait support
                let uses_native_async = if line_num > 0 {
                    lines[..line_num]
                        .iter()
                        .rev()
                        .take(5)
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
    violations
}
