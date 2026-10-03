//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::filters::LanguageId;
use crate::pattern_registry::required_pattern;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};
use mcb_utils::constants::validate::{CONTEXT_PREVIEW_LENGTH, TEST_DIR_FRAGMENT};

use super::violation::AsyncViolation;
use mcb_utils::constants::validate::BACKGROUND_FN_PATTERNS;

/// Returns `true` when `line` contains a `tokio::spawn` whose handle is neither
/// assigned, awaited, nor explicitly discarded with `let _`.
fn is_unassigned_spawn(
    line: &str,
    spawn_pattern: &regex::Regex,
    assigned_spawn_pattern: &regex::Regex,
) -> bool {
    let has_spawn = spawn_pattern.is_match(line);
    let has_followup =
        assigned_spawn_pattern.is_match(line) || line.contains(".await") || line.contains("let _");
    has_spawn && !has_followup
}

/// Detect spawn without await patterns
pub fn validate_spawn_patterns(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    // Pattern: tokio::spawn without assigning to variable or awaiting
    let spawn_pattern = required_pattern("ASYNC001.tokio_spawn")?;
    let assigned_spawn_pattern = required_pattern("ASYNC001.assigned_spawn")?;
    let fn_pattern = required_pattern("ASYNC001.fn_decl")?;
    let is_background_fn = |name: &str| BACKGROUND_FN_PATTERNS.iter().any(|p| name.contains(p));

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path.to_str().is_some_and(|s| s.contains(TEST_DIR_FRAGMENT)) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut current_fn_name = String::new();

        crate::validators::for_each_non_test_non_comment_line(
            &content,
            |line_num, line, trimmed| {
                // Track current function name
                if let Some(cap) = fn_pattern.captures(line) {
                    current_fn_name = cap.get(1).map_or("", |m| m.as_str()).to_lowercase();
                }

                if is_unassigned_spawn(line, spawn_pattern, assigned_spawn_pattern)
                    && !is_background_fn(&current_fn_name)
                {
                    violations.push(AsyncViolation::UnawaitedSpawn {
                        file: path.clone(),
                        line: line_num + 1,
                        context: trimmed.chars().take(CONTEXT_PREVIEW_LENGTH).collect(),
                        severity: Severity::Info,
                    });
                }
            },
        );

        Ok(())
    })?;

    Ok(violations)
}
