//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use rust_code_analysis::SpaceKind;

use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::ValidationConfigExt;
use crate::ast::rca_helpers;
use crate::filters::LanguageId;
use crate::run_context::ValidationRunContext;
use crate::scan::for_each_scan_file;
use crate::utils::source::structs_seem_related;
use crate::validators::solid::violation::SolidViolation;
use mcb_utils::constants::validate::MAX_UNRELATED_STRUCTS_PER_FILE;

/// SRP: Check for impl blocks that are too large (via RCA AST metrics).
///
/// # Errors
/// Returns an error if file scanning or reading fails.
pub fn validate_srp(
    config: &ValidationConfig,
    max_struct_lines: usize,
) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();
    mcb_domain::debug!("solid_srp", "Checking impl block sizes (SRP)");

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            if !entry.absolute_path.starts_with(&src_dir) {
                return Ok(());
            }
            let ctx = ValidationRunContext::active_or_build(config)?;
            let content = ctx
                .read_cached(&entry.absolute_path)
                .map_err(|e| crate::ValidationError::Config(e.to_string()))?;
            let Some(root) = rca_helpers::parse_file_spaces(&entry.absolute_path, &content) else {
                return Ok(());
            };

            scan_srp_file(
                &entry.absolute_path,
                &root,
                &content,
                max_struct_lines,
                &mut violations,
            );
            Ok(())
        })?;
    }

    Ok(violations)
}

/// Flag oversized impl blocks and files with multiple unrelated structs.
fn scan_srp_file(
    file: &std::path::Path,
    root: &rust_code_analysis::FuncSpace,
    content: &str,
    max_struct_lines: usize,
    violations: &mut Vec<SolidViolation>,
) {
    for space in rca_helpers::collect_spaces_of_kind(root, content, SpaceKind::Impl) {
        let name = space.name.as_deref().unwrap_or("");
        let sloc = space.metrics.loc.sloc().round() as usize;
        if sloc > max_struct_lines {
            violations.push(SolidViolation::TooManyResponsibilities {
                file: file.to_path_buf(),
                line: space.start_line,
                item_type: "impl".to_owned(),
                item_name: name.to_owned(),
                line_count: sloc,
                max_allowed: max_struct_lines,
                suggestion: "Consider splitting into smaller, focused impl blocks".to_owned(),
                severity: Severity::Warning,
            });
        }
    }

    let struct_names: Vec<String> =
        rca_helpers::collect_spaces_of_kind(root, content, SpaceKind::Struct)
            .iter()
            .filter_map(|s| s.name.clone())
            .collect();
    if struct_names.len() > MAX_UNRELATED_STRUCTS_PER_FILE && !structs_seem_related(&struct_names) {
        violations.push(SolidViolation::MultipleUnrelatedStructs {
            file: file.to_path_buf(),
            struct_names,
            suggestion: "Consider splitting into separate modules".to_owned(),
            severity: Severity::Info,
        });
    }
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
    mcb_domain::debug!("solid_srp", "Checking impl method counts");

    for crate_dir in config.get_source_dirs()? {
        let src_dir = crate_dir.join("src");
        if !src_dir.exists() {
            continue;
        }

        for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
            if !entry.absolute_path.starts_with(&src_dir) {
                return Ok(());
            }
            let ctx = ValidationRunContext::active_or_build(config)?;
            let content = ctx
                .read_cached(&entry.absolute_path)
                .map_err(|e| crate::ValidationError::Config(e.to_string()))?;
            let Some(root) = rca_helpers::parse_file_spaces(&entry.absolute_path, &content) else {
                return Ok(());
            };

            scan_impl_method_counts(
                &entry.absolute_path,
                &root,
                &content,
                max_impl_methods,
                &mut violations,
            );
            Ok(())
        })?;
    }

    Ok(violations)
}

/// Flag impl blocks whose method count exceeds `max_impl_methods`.
fn scan_impl_method_counts(
    file: &std::path::Path,
    root: &rust_code_analysis::FuncSpace,
    content: &str,
    max_impl_methods: usize,
    violations: &mut Vec<SolidViolation>,
) {
    for space in rca_helpers::collect_spaces_of_kind(root, content, SpaceKind::Impl) {
        let name = space.name.as_deref().unwrap_or("");
        let method_count =
            (space.metrics.nom.functions() + space.metrics.nom.closures()).round() as usize;
        if method_count > max_impl_methods {
            violations.push(SolidViolation::ImplTooManyMethods {
                file: file.to_path_buf(),
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
}
