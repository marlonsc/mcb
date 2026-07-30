//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#quality)
//!
use super::{QualityValidator, QualityViolation};
use crate::ast::rca_helpers;
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};
use mcb_utils::constants::validate::TEST_PATH_PATTERNS;

/// Checks that source files do not exceed the configured line count limit.
///
/// Uses RCA's aggregate SLOC metric for accurate source line counting.
///
/// # Errors
///
/// Returns an error if file scanning or content reading fails.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            if let Some(violation) = check_file_size(validator, &entry.absolute_path)? {
                violations.push(violation);
            }
            Ok(())
        },
    )?;

    Ok(violations)
}

/// Returns a `FileTooLarge` violation when `path` is an includable Rust file
/// whose SLOC exceeds the configured maximum, else `None`.
///
/// # Errors
///
/// Returns an error if the file content cannot be read.
fn check_file_size(
    validator: &QualityValidator,
    path: &std::path::Path,
) -> Result<Option<QualityViolation>> {
    let path_str = path.to_str();
    let excluded = path.extension().is_none_or(|ext| ext != "rs")
        || validator.config.should_exclude(path)
        || path_str.is_some_and(|s| TEST_PATH_PATTERNS.iter().any(|p| s.contains(p)))
        || !path.exists();
    if excluded {
        return Ok(None);
    }

    let Some(path_str) = path_str else {
        return Ok(None);
    };

    // Skip paths excluded in configuration (e.g., large vector store implementations).
    if validator
        .excluded_paths
        .iter()
        .any(|excluded| path_str.contains(excluded.as_str()))
    {
        return Ok(None);
    }

    let ctx = ValidationRunContext::active_or_build(&validator.config)?;
    let content = ctx
        .read_cached(path)
        .map_err(|e| crate::ValidationError::Config(e.to_string()))?;

    // Use RCA's SLOC for accurate source line counting (excludes blanks/comments).
    let line_count = rca_helpers::parse_file_spaces(path, &content).map_or_else(
        || content.lines().count(), // fallback for unsupported languages
        |root| root.metrics.loc.sloc().round() as usize,
    );

    Ok(
        (line_count > validator.max_file_lines).then(|| QualityViolation::FileTooLarge {
            file: path.to_path_buf(),
            lines: line_count,
            max_allowed: validator.max_file_lines,
            severity: Severity::Warning,
        }),
    )
}
