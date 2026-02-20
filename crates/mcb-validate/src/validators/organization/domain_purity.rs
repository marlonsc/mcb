//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use super::constants::{
    DOMAIN_ALLOWED_METHODS, DOMAIN_ALLOWED_PREFIXES, DOMAIN_CRATE_PATH, PORTS_DIR_PATH,
};
use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::{for_each_scan_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};

/// Verifies that the domain layer contains only trait definitions and data structures, free of implementation logic.
///
/// Ensures that the domain layer remains pure and free of side effects or business logic implementation,
/// permitting only:
/// - Trait definitions.
/// - Struct/enum definitions.
/// - Constructors, accessors, and derived implementations.
///
/// # Errors
///
/// Returns an error if file scanning or reading fails.
pub fn validate_domain_traits_only(
    config: &ValidationConfig,
) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Pattern for impl blocks with methods
    let impl_block_pattern = compile_regex(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s*\{")?;
    let method_pattern = compile_regex(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")?;

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, src_dir| {
        let Some(path_str) = entry.absolute_path.to_str() else {
            return Ok(());
        };

        if !src_dir
            .to_str()
            .is_some_and(|s| s.contains(DOMAIN_CRATE_PATH))
            || is_test_path(path_str)
            || path_str.contains(PORTS_DIR_PATH)
        {
            return Ok(());
        }

        let content = std::fs::read_to_string(&entry.absolute_path)?;

        let mut in_impl_block = false;
        let mut impl_name = String::new();
        let mut brace_depth = 0;
        let mut impl_start_brace = 0;

        crate::validators::for_each_non_test_non_comment_line(
            &content,
            |line_num, line, trimmed| {
                let line_num = line_num + 1;

                if let Some(cap) = impl_block_pattern.captures(line)
                    && !trimmed.contains("trait ")
                {
                    in_impl_block = true;
                    impl_name = cap.get(1).map_or("", |m| m.as_str()).to_owned();
                    impl_start_brace = brace_depth;
                }

                brace_depth +=
                    i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(0);
                brace_depth -=
                    i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(0);

                if in_impl_block && brace_depth <= impl_start_brace {
                    in_impl_block = false;
                }

                if in_impl_block && let Some(cap) = method_pattern.captures(line) {
                    let method_name = cap.get(1).map_or("", |m| m.as_str());
                    let allowed_method = DOMAIN_ALLOWED_METHODS.contains(&method_name)
                        || DOMAIN_ALLOWED_PREFIXES
                            .iter()
                            .any(|p| method_name.starts_with(p))
                        || (line.contains("&self") && !line.contains("&mut self"));

                    if !allowed_method {
                        violations.push(OrganizationViolation::DomainLayerImplementation {
                            file: entry.absolute_path.clone(),
                            line: line_num,
                            impl_type: "method".to_owned(),
                            type_name: format!("{impl_name}::{method_name}"),
                            severity: Severity::Info,
                        });
                    }
                }
            },
        );

        Ok(())
    })?;

    Ok(violations)
}
