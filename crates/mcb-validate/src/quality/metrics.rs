use super::{QualityValidator, QualityViolation};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity};

/// Checks that source files do not exceed the configured line count limit.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    for_each_scan_rs_path(&validator.config, false, |path, _src_dir| {
        if path.extension().is_none_or(|ext| ext != "rs")
            || validator.config.should_exclude(path)
            || path.to_string_lossy().contains("/tests/")
            || path.to_string_lossy().contains("/target/")
            || path.to_string_lossy().ends_with("_test.rs")
            || !path.exists()
        {
            return Ok(());
        }

        let path_str = path.to_string_lossy();

        // Skip paths excluded in configuration (e.g., large vector store implementations)
        if validator
            .excluded_paths
            .iter()
            .any(|excluded| path_str.contains(excluded.as_str()))
        {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let line_count = content.lines().count();

        if line_count > validator.max_file_lines {
            violations.push(QualityViolation::FileTooLarge {
                file: path.to_path_buf(),
                lines: line_count,
                max_allowed: validator.max_file_lines,
                severity: Severity::Warning,
            });
        }

        Ok(())
    })?;

    Ok(violations)
}
