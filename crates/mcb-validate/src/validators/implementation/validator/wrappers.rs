use std::path::PathBuf;

use regex::{Match, Regex};

use super::super::violation::ImplementationViolation;
use super::utils::{extract_functions_with_body, non_test_lines, required_pattern};
use crate::Result;
use crate::traits::violation::Severity;

pub fn validate_pass_through_wrappers(
    files: &[(PathBuf, String)],
    fn_pattern: &Regex,
) -> Result<Vec<ImplementationViolation>> {
    let passthrough_pattern = required_pattern("IMPL001.passthrough")?;
    let impl_pattern = required_pattern("IMPL001.impl_decl")?;

    let mut violations = Vec::new();

    for (file_path, content) in files {
        let fname = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if fname.contains("adapter") || fname.contains("wrapper") {
            continue;
        }

        let lines: Vec<&str> = content.lines().collect();
        let non_test = non_test_lines(&lines);

        // Track current impl block
        let mut current_struct_name = String::new();
        for func in extract_functions_with_body(
            Some(fn_pattern),
            Some(impl_pattern),
            &non_test,
            &mut current_struct_name,
        ) {
            if func.meaningful_body.len() != 1 {
                continue;
            }
            if let Some(cap) = passthrough_pattern.captures(&func.meaningful_body[0]) {
                let field = cap.get(1).map_or("", |m: Match| m.as_str());
                let method = cap.get(2).map_or("", |m: Match| m.as_str());
                if method == func.name || method.starts_with(&func.name) {
                    violations.push(ImplementationViolation::PassThroughWrapper {
                        file: file_path.clone(),
                        line: func.start_line,
                        struct_name: current_struct_name.clone(),
                        method_name: func.name.clone(),
                        delegated_to: format!("self.{field}.{method}()"),
                        severity: Severity::Info,
                    });
                }
            }
        }
    }
    Ok(violations)
}
