//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use rust_code_analysis::SpaceKind;

use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::ast::rca_helpers;
use crate::constants::solid::MAX_UNRELATED_STRUCTS_PER_FILE;
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::utils::source::structs_seem_related;
use crate::validators::solid::violation::SolidViolation;

/// SRP: Check for impl blocks that are too large (via RCA AST metrics).
///
/// # Errors
/// Returns an error if file scanning or reading fails.
pub fn validate_srp(
    config: &ValidationConfig,
    max_struct_lines: usize,
) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            if !entry.absolute_path.starts_with(&src_dir) {
                return Ok(());
            }
            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let Some(root) = rca_helpers::parse_file_spaces(&entry.absolute_path, &content) else {
                return Ok(());
            };

            // Check impl blocks via RCA SpaceKind::Impl
            for space in rca_helpers::collect_spaces_of_kind(&root, &content, SpaceKind::Impl) {
                let name = space.name.as_deref().unwrap_or("");
                let sloc = space.metrics.loc.sloc().round() as usize;

                if sloc > max_struct_lines {
                    violations.push(SolidViolation::TooManyResponsibilities {
                        file: entry.absolute_path.clone(),
                        line: space.start_line,
                        item_type: "impl".to_owned(),
                        item_name: name.to_owned(),
                        line_count: sloc,
                        max_allowed: max_struct_lines,
                        suggestion: "Consider splitting into smaller, focused impl blocks"
                            .to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }

            // Check for multiple unrelated structs via RCA SpaceKind::Struct
            let struct_names: Vec<String> =
                rca_helpers::collect_spaces_of_kind(&root, &content, SpaceKind::Struct)
                    .iter()
                    .filter_map(|s| s.name.clone())
                    .collect();

            if struct_names.len() > MAX_UNRELATED_STRUCTS_PER_FILE
                && !structs_seem_related(&struct_names)
            {
                violations.push(SolidViolation::MultipleUnrelatedStructs {
                    file: entry.absolute_path.clone(),
                    struct_names,
                    suggestion: "Consider splitting into separate modules".to_owned(),
                    severity: Severity::Info,
                });
            }

            Ok(())
        })?;
    }

    Ok(violations)
}

/// SRP: Check for impl blocks with too many methods (via RCA NOM metric).
///
/// # Errors
/// Returns an error if file scanning or reading fails.
pub fn validate_impl_method_count(
    config: &ValidationConfig,
    max_impl_methods: usize,
) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            if !entry.absolute_path.starts_with(&src_dir) {
                return Ok(());
            }
            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let Some(root) = rca_helpers::parse_file_spaces(&entry.absolute_path, &content) else {
                return Ok(());
            };

            for space in rca_helpers::collect_spaces_of_kind(&root, &content, SpaceKind::Impl) {
                let name = space.name.as_deref().unwrap_or("");
                let method_count =
                    (space.metrics.nom.functions() + space.metrics.nom.closures()).round() as usize;

                if method_count > max_impl_methods {
                    violations.push(SolidViolation::ImplTooManyMethods {
                        file: entry.absolute_path.clone(),
                        line: space.start_line,
                        type_name: name.to_owned(),
                        method_count,
                        max_allowed: max_impl_methods,
                        suggestion:
                            "Consider splitting into smaller, focused impl blocks or extracting to traits"
                                .to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        })?;
    }

    Ok(violations)
}
