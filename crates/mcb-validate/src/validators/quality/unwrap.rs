//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
use super::constants::{
    COMMENT_SEARCH_RADIUS, IGNORE_HINT_KEYWORDS, LOCK_POISONING_STRINGS, SAFETY_COMMENT_MARKERS,
};
use super::{QualityValidator, QualityViolation};
use crate::ast::UnwrapDetector;
use crate::constants::common::TEST_PATH_PATTERNS;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

fn has_ignore_hint(line: &str, violation_type: &str) -> bool {
    line.contains(&format!("mcb-validate-ignore: {violation_type}"))
}

fn should_skip_source_file(path: &std::path::Path) -> bool {
    let Some(path_str) = path.to_str() else {
        return true;
    };
    path.extension().is_none_or(|ext| ext != "rs")
        || TEST_PATH_PATTERNS.iter().any(|p| path_str.contains(p))
}

fn has_ignore_nearby(lines: &[&str], detection_line: usize) -> bool {
    if detection_line == 0 || detection_line > lines.len() {
        return false;
    }

    let start = detection_line.saturating_sub(COMMENT_SEARCH_RADIUS + 1);
    let end = (detection_line + 2).min(lines.len() - 1);
    (start..=end).any(|idx| {
        let line = lines[idx];
        IGNORE_HINT_KEYWORDS
            .iter()
            .any(|k| has_ignore_hint(line, k))
    })
}

fn is_lock_poisoning_expect(method: &str, line_content: &str) -> bool {
    method == "expect"
        && LOCK_POISONING_STRINGS
            .iter()
            .any(|s| line_content.contains(s))
}

fn should_skip_detection(
    detection: &crate::ast::unwrap_detector::UnwrapDetection,
    lines: &[&str],
) -> bool {
    if detection.in_test || detection.line == 0 || detection.line > lines.len() {
        return true;
    }

    let line_content = lines[detection.line - 1];
    SAFETY_COMMENT_MARKERS
        .iter()
        .any(|m| line_content.contains(m))
        || has_ignore_nearby(lines, detection.line - 1)
        || is_lock_poisoning_expect(&detection.method, line_content)
}

fn push_violation(
    violations: &mut Vec<QualityViolation>,
    file: &std::path::Path,
    detection: crate::ast::unwrap_detector::UnwrapDetection,
) {
    match detection.method.as_str() {
        "unwrap" => {
            violations.push(QualityViolation::UnwrapInProduction {
                file: file.to_path_buf(),
                line: detection.line,
                context: detection.context,
                severity: Severity::Warning,
            });
        }
        "expect" => {
            violations.push(QualityViolation::ExpectInProduction {
                file: file.to_path_buf(),
                line: detection.line,
                context: detection.context,
                severity: Severity::Warning,
            });
        }
        other => {
            tracing::debug!(
                method = other,
                file = %file.display(),
                line = detection.line,
                "unhandled detection method type"
            );
        }
    }
}

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
            if should_skip_source_file(&entry.absolute_path) {
                return Ok(());
            }
            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let lines: Vec<&str> = content.lines().collect();
            let detections = detector.detect_in_file(&entry.absolute_path)?;

            for detection in detections {
                if should_skip_detection(&detection, &lines) {
                    continue;
                }
                push_violation(&mut violations, &entry.absolute_path, detection);
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
