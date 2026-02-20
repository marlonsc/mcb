//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#refactoring)
//!
use std::path::Path;

use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

use super::RefactoringValidator;
use super::violation::RefactoringViolation;

/// Check for mod declarations that reference non-existent files
pub fn validate_mod_declarations(
    validator: &RefactoringValidator,
) -> Result<Vec<RefactoringViolation>> {
    let mut violations = Vec::new();
    let mod_pattern = compile_regex(r"(?:pub\s+)?mod\s+([a-z_][a-z0-9_]*)(?:\s*;)")?;

    for src_dir in validator.config.get_scan_dirs()? {
        if validator.should_skip_crate(&src_dir) {
            continue;
        }

        for_each_scan_file(
            &validator.config,
            Some(LanguageId::Rust),
            false,
            |entry, candidate_src_dir| {
                let path = &entry.absolute_path;
                if candidate_src_dir != src_dir {
                    return Ok(());
                }

                let parent_dir = path.parent().unwrap_or(Path::new("."));
                let content = std::fs::read_to_string(path)?;
                let file_stem = path.file_stem().and_then(|s| s.to_str());
                let module_decl_exists = |mod_name: &str| {
                    let direct = [
                        parent_dir.join(format!("{mod_name}.rs")),
                        parent_dir.join(mod_name).join("mod.rs"),
                    ];
                    let nested = file_stem.map(|stem| {
                        [
                            parent_dir.join(stem).join(format!("{mod_name}.rs")),
                            parent_dir.join(stem).join(mod_name).join("mod.rs"),
                        ]
                    });

                    direct.into_iter().any(|p| p.exists())
                        || nested.is_some_and(|paths| paths.into_iter().any(|p| p.exists()))
                };

                for (line_num, line) in content.lines().enumerate() {
                    if let Some(cap) = mod_pattern.captures(line) {
                        let mod_name = cap.get(1).map_or("", |m| m.as_str());

                        if !module_decl_exists(mod_name) {
                            violations.push(RefactoringViolation::DeletedModuleReference {
                                referencing_file: path.clone(),
                                line: line_num + 1,
                                deleted_module: mod_name.to_owned(),
                                severity: Severity::Warning,
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
