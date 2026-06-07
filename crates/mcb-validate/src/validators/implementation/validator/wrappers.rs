//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::{Path, PathBuf};

use regex::{Match, Regex};

use super::super::violation::ImplementationViolation;
use crate::Result;
use crate::pattern_registry::required_pattern;
use crate::utils::source::{extract_functions_with_body, non_test_lines};
use mcb_domain::ports::validation::Severity;

/// Detects single-line pass-through wrapper methods that merely delegate to a
/// field's identically named method and reports each as a `PassThroughWrapper`
/// violation. Files named `adapter`/`wrapper` are skipped as intentional.
///
/// # Errors
///
/// Returns an error if the IMPL001 pass-through or impl-declaration pattern is
/// missing from the pattern registry.
pub fn validate_pass_through_wrappers(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let passthrough_pattern = required_pattern("IMPL001.passthrough")?;
    let impl_pattern = required_pattern("IMPL001.impl_decl")?;

    let mut violations = Vec::new();

    for (file_path, content) in files {
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if ["adapter", "wrapper"].iter().any(|kw| fname.contains(kw)) {
            continue;
        }
        collect_pass_through_wrappers(
            file_path,
            content,
            fn_pattern,
            impl_pattern,
            passthrough_pattern,
            &mut violations,
        );
    }
    Ok(violations)
}

/// Push a `PassThroughWrapper` violation for each single-line method in
/// `content` that simply delegates to a field method of the same (or prefixed)
/// name.
fn collect_pass_through_wrappers(
    file_path: &Path,
    content: &str,
    fn_pattern: &Regex,
    impl_pattern: &Regex,
    passthrough_pattern: &Regex,
    violations: &mut Vec<ImplementationViolation>,
) {
    let non_test = non_test_lines(&content.lines().collect::<Vec<_>>());
    let mut current_struct_name = String::new();
    for func in extract_functions_with_body(
        Some(fn_pattern),
        Some(impl_pattern),
        &non_test,
        &mut current_struct_name,
    ) {
        let matched = (func.meaningful_body.len() == 1)
            .then(|| passthrough_pattern.captures(&func.meaningful_body[0]))
            .flatten();
        let Some(cap) = matched else {
            continue;
        };
        let field = cap.get(1).map_or("", |m: Match| m.as_str());
        let method = cap.get(2).map_or("", |m: Match| m.as_str());
        if method == func.name || method.starts_with(&func.name) {
            violations.push(ImplementationViolation::PassThroughWrapper {
                file: file_path.to_path_buf(),
                line: func.start_line,
                struct_name: current_struct_name.clone(),
                method_name: func.name.clone(),
                delegated_to: format!("self.{field}.{method}()"),
                severity: Severity::Info,
            });
        }
    }
}
