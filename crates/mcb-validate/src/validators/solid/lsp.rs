//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::utils::source::for_each_rust_file;
use crate::validators::solid::violation::SolidViolation;
use std::path::Path;

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
        violations.extend(collect_lsp_violations_for_file(
            &path,
            &lines,
            impl_for_pattern,
            fn_pattern,
            panic_todo_pattern,
        ));
        Ok(())
    })?;

    Ok(violations)
}

fn collect_lsp_violations_for_file(
    path: &Path,
    lines: &[&str],
    impl_for_pattern: &regex::Regex,
    fn_pattern: &regex::Regex,
    panic_todo_pattern: &regex::Regex,
) -> Vec<SolidViolation> {
    lines
        .iter()
        .enumerate()
        .filter_map(|(line_num, line)| {
            let captures = impl_for_pattern.captures(line)?;
            let trait_name = captures.get(1).map_or("", |m| m.as_str());
            let impl_name = captures.get(2).map_or("", |m| m.as_str());
            let (block_lines, _) = crate::scan::extract_balanced_block(lines, line_num)?;
            let input = BlockScanInput {
                path,
                impl_start_line: line_num,
                block_lines: &block_lines,
                trait_name,
                impl_name,
                fn_pattern,
                panic_todo_pattern,
            };
            Some(collect_block_partial_impl_violations(&input))
        })
        .flatten()
        .collect()
}

struct BlockScanInput<'a> {
    path: &'a Path,
    impl_start_line: usize,
    block_lines: &'a [&'a str],
    trait_name: &'a str,
    impl_name: &'a str,
    fn_pattern: &'a regex::Regex,
    panic_todo_pattern: &'a regex::Regex,
}

fn collect_block_partial_impl_violations(input: &BlockScanInput<'_>) -> Vec<SolidViolation> {
    let mut current_method: Option<(String, usize)> = None;
    let mut violations = Vec::new();

    for (offset, impl_line) in input.block_lines.iter().enumerate() {
        if let Some(next_method) = parse_method_signature(
            input.fn_pattern,
            impl_line,
            input.impl_start_line + offset + 1,
        ) {
            current_method = Some(next_method);
        }

        if input.panic_todo_pattern.is_match(impl_line)
            && let Some((method_name, method_line)) = current_method.take()
        {
            violations.push(SolidViolation::PartialTraitImplementation {
                file: input.path.to_path_buf(),
                line: method_line,
                impl_name: format!("{}::{}", input.impl_name, input.trait_name),
                method_name,
                severity: Severity::Warning,
            });
        }
    }

    violations
}

fn parse_method_signature(
    fn_pattern: &regex::Regex,
    impl_line: &str,
    line_number: usize,
) -> Option<(String, usize)> {
    let captures = fn_pattern.captures(impl_line)?;
    let method_name = captures.get(1).map_or("", |m| m.as_str());
    Some((method_name.to_owned(), line_number))
}
