use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::utils::source::for_each_rust_file;
use crate::validators::solid::violation::SolidViolation;

/// LSP: Check for partial trait implementations (panic!/todo! in trait methods).
///
/// # Errors
/// Returns an error if pattern compilation fails.
pub fn validate_lsp(config: &ValidationConfig) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();
    let impl_for_pattern = required_pattern("SOLID002.impl_for_decl")?;
    let fn_pattern = required_pattern("SOLID002.fn_decl")?;
    let panic_todo_pattern = required_pattern("SOLID003.panic_macros")?;

    for_each_rust_file(config, |path, lines| {
        for (line_num, line) in lines.iter().enumerate() {
            if let Some(cap) = impl_for_pattern.captures(line) {
                let trait_name = cap.get(1).map_or("", |m| m.as_str());
                let impl_name = cap.get(2).map_or("", |m| m.as_str());

                if let Some((block_lines, _)) =
                    crate::scan::extract_balanced_block(&lines, line_num)
                {
                    let mut current_method: Option<(String, usize)> = None;

                    for (idx, impl_line) in block_lines.iter().enumerate() {
                        if let Some(fn_cap) = fn_pattern.captures(impl_line) {
                            let method_name = fn_cap.get(1).map_or("", |m| m.as_str());
                            current_method = Some((method_name.to_owned(), line_num + idx + 1));
                        }

                        if let Some((ref method_name, method_line)) = current_method
                            && panic_todo_pattern.is_match(impl_line)
                        {
                            violations.push(SolidViolation::PartialTraitImplementation {
                                file: path.clone(),
                                line: method_line,
                                impl_name: format!("{impl_name}::{trait_name}"),
                                method_name: method_name.clone(),
                                severity: Severity::Warning,
                            });
                            current_method = None;
                        }
                    }
                }
            }
        }
        Ok(())
    })?;

    Ok(violations)
}
