use crate::Result;
use crate::Severity;
use crate::ValidationConfig;
use crate::pattern_registry::required_pattern;
use crate::validators::solid::utils::{
    count_block_lines, count_matches_in_block, for_each_rust_file, scan_decl_blocks,
    structs_seem_related,
};
use crate::validators::solid::violation::SolidViolation;

/// SRP: Check for structs/impls that are too large
///
/// # Errors
/// Returns an error if patterns fail to compile or file reading fails.
pub fn validate_srp(
    config: &ValidationConfig,
    max_struct_lines: usize,
) -> Result<Vec<SolidViolation>> {
    let mut violations = Vec::new();
    let impl_pattern = required_pattern("SOLID002.impl_decl")?;
    let struct_pattern = required_pattern("SOLID002.struct_decl")?;

    for_each_rust_file(config, |path, lines| {
        let mut structs_in_file: Vec<(String, usize)> = Vec::new();

        for (line_num, line) in lines.iter().enumerate() {
            if let Some(cap) = struct_pattern.captures(line) {
                let name = cap.get(1).map_or("", |m| m.as_str());
                structs_in_file.push((name.to_owned(), line_num + 1));
            }

            if let Some(cap) = impl_pattern.captures(line) {
                let name = cap.get(1).or(cap.get(2)).map_or("", |m| m.as_str());
                let block_lines = count_block_lines(&lines, line_num);

                if block_lines > max_struct_lines {
                    violations.push(SolidViolation::TooManyResponsibilities {
                        file: path.clone(),
                        line: line_num + 1,
                        item_type: "impl".to_owned(),
                        item_name: name.to_owned(),
                        line_count: block_lines,
                        max_allowed: max_struct_lines,
                        suggestion: "Consider splitting into smaller, focused impl blocks"
                            .to_owned(),
                        severity: Severity::Warning,
                    });
                }
            }
        }

        if structs_in_file.len() > 5 {
            let struct_names: Vec<String> =
                structs_in_file.iter().map(|(n, _)| n.clone()).collect();

            if !structs_seem_related(&struct_names) {
                violations.push(SolidViolation::MultipleUnrelatedStructs {
                    file: path.clone(),
                    struct_names,
                    suggestion: "Consider splitting into separate modules".to_owned(),
                    severity: Severity::Info,
                });
            }
        }
        Ok(())
    })?;

    Ok(violations)
}

/// SRP: Check for impl blocks with too many methods
///
/// # Errors
/// Returns an error if pattern compilation fails.
#[allow(dead_code)]
pub fn validate_impl_method_count(
    config: &ValidationConfig,
    max_impl_methods: usize,
) -> Result<Vec<SolidViolation>> {
    let impl_pattern = required_pattern("SOLID003.impl_only_decl")?;
    let fn_pattern = required_pattern("SOLID002.fn_decl")?;

    scan_decl_blocks(
        config,
        impl_pattern,
        fn_pattern,
        count_matches_in_block,
        max_impl_methods,
        |file, line, type_name, method_count, max_allowed| SolidViolation::ImplTooManyMethods {
            file,
            line,
            type_name: type_name.to_owned(),
            method_count,
            max_allowed,
            suggestion:
                "Consider splitting into smaller, focused impl blocks or extracting to traits"
                    .to_owned(),
            severity: Severity::Warning,
        },
    )
}
