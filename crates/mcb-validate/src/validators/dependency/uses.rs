//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
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

                crate::validators::for_each_non_test_non_comment_line(
                    &content,
                    |line_num, line, _trimmed| {
                        if line.trim().starts_with("/*") || line.contains('"') {
                            return;
                        }
                        violations.extend(use_pattern.captures_iter(line).filter_map(|cap| {
                            let used_crate_kebab =
                                cap.get(1).map_or("", |m| m.as_str()).replace('_', "-");
                            (used_crate_kebab != *crate_name
                                && !allowed.contains(&used_crate_kebab))
                            .then(|| DependencyViolation::ForbiddenUseStatement {
                                crate_name: crate_name.clone(),
                                forbidden_dep: used_crate_kebab,
                                file: path.clone(),
                                line: line_num + 1,
                                context: line.trim().to_owned(),
                                severity: Severity::Error,
                            })
                        }));
                    },
                );

                Ok(())
            },
        )?;
    }

    Ok(violations)
}
