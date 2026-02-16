use crate::constants::common::COMMENT_PREFIX;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_file_under_root;
use crate::{Result, Severity};

use super::DependencyValidator;
use super::violation::DependencyViolation;

/// Validate no forbidden use statements in source code
pub fn validate_use_statements(
    validator: &DependencyValidator,
) -> Result<Vec<DependencyViolation>> {
    let mut violations = Vec::new();
    let use_pattern = compile_regex(r"use\s+(mcb_[a-z_]+)")?;

    for (crate_name, allowed) in &validator.allowed_deps {
        let crate_src = validator
            .config
            .workspace_root
            .join("crates")
            .join(crate_name)
            .join("src");

        if !crate_src.exists() {
            continue;
        }

        for_each_file_under_root(
            &validator.config,
            &crate_src,
            Some(LanguageId::Rust),
            |entry| {
                let path = &entry.absolute_path;
                let content = std::fs::read_to_string(path)?;

                for (line_num, line) in content.lines().enumerate() {
                    // Skip comments
                    let trimmed = line.trim();
                    if trimmed.starts_with(COMMENT_PREFIX) || trimmed.starts_with("/*") {
                        continue;
                    }

                    // Skip lines that are likely string literals (contain quotes)
                    if line.contains('"') {
                        continue;
                    }

                    for cap in use_pattern.captures_iter(line) {
                        let used_crate = cap.get(1).map_or("", |m| m.as_str());
                        let used_crate_kebab = used_crate.replace('_', "-");

                        // Skip self-references
                        if used_crate_kebab == *crate_name {
                            continue;
                        }

                        // Check if this dependency is allowed
                        if !allowed.contains(&used_crate_kebab) {
                            violations.push(DependencyViolation::ForbiddenUseStatement {
                                crate_name: crate_name.clone(),
                                forbidden_dep: used_crate_kebab,
                                file: path.clone(),
                                line: line_num + 1,
                                context: line.trim().to_owned(),
                                severity: Severity::Error,
                            });
                        }
                    }
                }

                Ok(())
            },
        )?;
    }

    Ok(violations)
}
