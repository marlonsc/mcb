use super::utils::has_ignore_hint;
use super::{QualityValidator, QualityViolation};
use crate::ast::UnwrapDetector;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

/// Scans production code for usage of `unwrap()` and `expect()` methods.
///
/// Uses AST-based detection to accurately identify method calls while ignoring
/// test files and allowed patterns.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let mut detector = UnwrapDetector::new()?;

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
            {
                return Ok(());
            }

            let Some(path_str) = entry.absolute_path.to_str() else {
                return Ok(());
            };
            if path_str.contains("/tests/")
                || path_str.contains("/target/")
                || path_str.ends_with("_test.rs")
                || path_str.ends_with("test.rs")
            {
                return Ok(());
            }

            // Use AST-based detection
            let detections = detector.detect_in_file(&entry.absolute_path)?;

            for detection in detections {
                // Skip detections in test modules
                if detection.in_test {
                    continue;
                }

                // Skip if context contains SAFETY justification or ignore hints
                // (checked via Regex since AST doesn't capture comments easily)
                let content = std::fs::read_to_string(&entry.absolute_path)?;
                let lines: Vec<&str> = content.lines().collect();
                if detection.line > 0 && detection.line <= lines.len() {
                    let line_content = lines[detection.line - 1];

                    // Check for safety comments
                    if line_content.contains("// SAFETY:") || line_content.contains("// safety:") {
                        continue;
                    }

                    // Check for ignore hints around the detection
                    let mut has_ignore = false;

                    // Check current line
                    if has_ignore_hint(line_content, "lock_poisoning_recovery")
                        || has_ignore_hint(line_content, "hardcoded_fallback")
                    {
                        has_ignore = true;
                    }

                    // Check previous lines (up to 3 lines before)
                    if !has_ignore && detection.line > 1 {
                        for i in 1..=3.min(detection.line - 1) {
                            let check_line = lines[detection.line - 1 - i];
                            if has_ignore_hint(check_line, "lock_poisoning_recovery")
                                || has_ignore_hint(check_line, "hardcoded_fallback")
                            {
                                has_ignore = true;
                                break;
                            }
                        }
                    }

                    // Check next lines (up to 3 lines after)
                    if !has_ignore && detection.line < lines.len() {
                        for i in 0..3.min(lines.len() - detection.line) {
                            let check_line = lines[detection.line + i];
                            if has_ignore_hint(check_line, "lock_poisoning_recovery")
                                || has_ignore_hint(check_line, "hardcoded_fallback")
                            {
                                has_ignore = true;
                                break;
                            }
                        }
                    }

                    if has_ignore {
                        continue;
                    }

                    // Skip cases where we're handling system-level errors appropriately
                    // (e.g., lock poisoning, which is a legitimate use of expect())
                    if detection.method == "expect"
                        && (line_content.contains("lock poisoned")
                            || line_content.contains("Lock poisoned")
                            || line_content.contains("poisoned")
                            || line_content.contains("Mutex poisoned"))
                    {
                        continue;
                    }
                }

                // Create appropriate violation based on method type
                match detection.method.as_str() {
                    "unwrap" => {
                        violations.push(QualityViolation::UnwrapInProduction {
                            file: entry.absolute_path.clone(),
                            line: detection.line,
                            context: detection.context,
                            severity: Severity::Warning,
                        });
                    }
                    "expect" => {
                        violations.push(QualityViolation::ExpectInProduction {
                            file: entry.absolute_path.clone(),
                            line: detection.line,
                            context: detection.context,
                            severity: Severity::Warning,
                        });
                    }
                    other => {
                        tracing::debug!(
                            method = other,
                            file = %entry.absolute_path.display(),
                            line = detection.line,
                            "unhandled detection method type"
                        );
                    }
                }
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
