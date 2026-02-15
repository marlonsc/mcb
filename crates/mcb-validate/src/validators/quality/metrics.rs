use super::{QualityValidator, QualityViolation};
use crate::constants::common::TEST_PATH_PATTERNS;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

/// Checks that source files do not exceed the configured line count limit.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            let path_str = entry.absolute_path.to_str();
            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
                || validator.config.should_exclude(&entry.absolute_path)
                || path_str.is_some_and(|s| TEST_PATH_PATTERNS.iter().any(|p| s.contains(p)))
                || !entry.absolute_path.exists()
            {
                return Ok(());
            }

            let Some(path_str) = path_str else {
                return Ok(());
            };

            // Skip paths excluded in configuration (e.g., large vector store implementations)
            if validator
                .excluded_paths
                .iter()
                .any(|excluded| path_str.contains(excluded.as_str()))
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let line_count = content.lines().count();

            if line_count > validator.max_file_lines {
                violations.push(QualityViolation::FileTooLarge {
                    file: entry.absolute_path.clone(),
                    lines: line_count,
                    max_allowed: validator.max_file_lines,
                    severity: Severity::Warning,
                });
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
