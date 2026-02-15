use crate::scan::for_each_file_under_root;
// use crate::traits::violation::ViolationCategory; // Removed unused import
use crate::{Result, Severity};

use std::path::{Path, PathBuf};

use crate::filters::LanguageId;

use super::DependencyValidator;
use super::violation::DependencyViolation;

/// Validate anti-bypass boundaries from config.
pub fn validate_bypass_boundaries(
    validator: &DependencyValidator,
) -> Result<Vec<DependencyViolation>> {
    let mut violations = Vec::new();
    let file_config = crate::config::FileConfig::load(&validator.config.workspace_root);

    for boundary in &file_config.rules.dependency.bypass_boundaries {
        let scan_root = validator.config.workspace_root.join(&boundary.scan_root);
        let allowed: Vec<&str> = boundary
            .allowed_files
            .iter()
            .map(std::string::String::as_str)
            .collect();
        let violation_id = boundary.violation_id.clone();
        let pattern = boundary.pattern.clone();

        scan_bypass_patterns(
            validator,
            &scan_root,
            |rel| !allowed.iter().any(|a| rel == Path::new(a)),
            &pattern,
            |file, line, context| match violation_id.as_str() {
                "DEP005" => DependencyViolation::CliBypassPath {
                    file,
                    line,
                    context,
                    severity: Severity::Error,
                },
                "DEP004" | _ => DependencyViolation::AdminBypassImport {
                    file,
                    line,
                    context,
                    severity: Severity::Error,
                },
            },
            &mut violations,
        )?;
    }

    Ok(violations)
}

fn scan_bypass_patterns<F, G>(
    validator: &DependencyValidator,
    scan_root: &Path,
    should_check_file: F,
    pattern: &str,
    make_violation: G,
    out: &mut Vec<DependencyViolation>,
) -> Result<()>
where
    F: Fn(&Path) -> bool,
    G: Fn(PathBuf, usize, String) -> DependencyViolation,
{
    if !scan_root.exists() {
        return Ok(());
    }

    for_each_file_under_root(
        &validator.config,
        scan_root,
        Some(LanguageId::Rust),
        |entry| {
            let path = &entry.absolute_path;
            let rel = path
                .strip_prefix(&validator.config.workspace_root)
                .unwrap_or(path);
            if !should_check_file(rel) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("//") {
                    continue;
                }
                if line.contains(pattern) {
                    out.push(make_violation(
                        path.clone(),
                        line_num + 1,
                        trimmed.to_owned(),
                    ));
                }
            }

            Ok(())
        },
    )?;

    Ok(())
}
