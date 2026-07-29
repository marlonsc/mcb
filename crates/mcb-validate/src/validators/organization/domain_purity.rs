//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use std::path::Path;

use regex::Regex;

use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::scan::{for_each_scan_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use mcb_utils::constants::validate::{
    DOMAIN_ALLOWED_METHODS, DOMAIN_ALLOWED_PREFIXES, DOMAIN_CRATE_PATH, PORTS_DIR,
};
use mcb_utils::utils::regex::compile_regex;

/// Regexes used to find impl blocks and their methods in domain source.
struct DomainImplPatterns {
    impl_block: Regex,
    method: Regex,
}

/// Tracks the current impl-block context while scanning a domain file.
#[derive(Default)]
struct ImplScanState {
    in_impl_block: bool,
    impl_name: String,
    brace_depth: i32,
    impl_start_brace: i32,
}

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
    let patterns = DomainImplPatterns {
        impl_block: compile_regex(r"impl\s+([A-Z][a-zA-Z0-9_]*)\s*\{")?,
        method: compile_regex(r"(?:pub\s+)?(?:async\s+)?fn\s+([a-z_][a-z0-9_]*)\s*\(")?,
    };

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, src_dir| {
        let Some(path_str) = entry.absolute_path.to_str() else {
            return Ok(());
        };
        let in_domain_non_port = src_dir
            .to_str()
            .is_some_and(|s| s.contains(DOMAIN_CRATE_PATH))
            && !is_test_path(path_str)
            && !path_str.contains(PORTS_DIR);
        if !in_domain_non_port {
            return Ok(());
        }

        let content = std::fs::read_to_string(&entry.absolute_path)?;
        scan_domain_impls(&entry.absolute_path, &content, &patterns, &mut violations);
        Ok(())
    })?;

    Ok(violations)
}

/// Scan one domain file, flagging non-pure methods inside `impl` blocks.
fn scan_domain_impls(
    file: &Path,
    content: &str,
    patterns: &DomainImplPatterns,
    violations: &mut Vec<OrganizationViolation>,
) {
    let mut state = ImplScanState::default();
    crate::validators::for_each_non_test_non_comment_line(content, |line_num, line, trimmed| {
        if let Some(cap) = patterns.impl_block.captures(line)
            && !trimmed.contains("trait ")
        {
            state.in_impl_block = true;
            state.impl_name = cap.get(1).map_or("", |m| m.as_str()).to_owned();
            state.impl_start_brace = state.brace_depth;
        }

        state.brace_depth += i32::try_from(line.matches('{').count()).unwrap_or(0);
        state.brace_depth -= i32::try_from(line.matches('}').count()).unwrap_or(0);
        state.in_impl_block &= state.brace_depth > state.impl_start_brace;

        if state.in_impl_block
            && let Some(violation) = disallowed_domain_method(
                file,
                line,
                line_num + 1,
                &state.impl_name,
                &patterns.method,
            )
        {
            violations.push(violation);
        }
    });
}

/// Returns a `DomainLayerImplementation` violation if `line` declares a method
/// that is not a permitted constructor/accessor/derived form, else `None`.
fn disallowed_domain_method(
    file: &Path,
    line: &str,
    line_num: usize,
    impl_name: &str,
    method_pattern: &Regex,
) -> Option<OrganizationViolation> {
    let cap = method_pattern.captures(line)?;
    let method_name = cap.get(1).map_or("", |m| m.as_str());
    let allowed = DOMAIN_ALLOWED_METHODS.contains(&method_name)
        || DOMAIN_ALLOWED_PREFIXES
            .iter()
            .any(|p| method_name.starts_with(p))
        || (line.contains("&self") && !line.contains("&mut self"));

    (!allowed).then(|| OrganizationViolation::DomainLayerImplementation {
        file: file.to_path_buf(),
        line: line_num,
        impl_type: "method".to_owned(),
        type_name: format!("{impl_name}::{method_name}"),
        severity: Severity::Info,
    })
}
