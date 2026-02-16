use super::{QualityValidator, QualityViolation};
use crate::constants::common::TEST_DIR_FRAGMENT;
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};
use regex::Regex;

/// Scans for and reports usage of `allow(dead_code)` attributes.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    let dead_code_pattern = compile_regex(r"#\[allow\([^\)]*dead_code[^\)]*\)\]")?;
    let struct_pattern = compile_regex(r"pub\s+struct\s+(\w+)")?;
    let fn_pattern = compile_regex(r"(?:pub\s+)?fn\s+(\w+)")?;
    let field_pattern = compile_regex(r"(?:pub\s+)?(\w+):\s+")?;

    for_each_scan_file(
        &validator.config,
        Some(LanguageId::Rust),
        false,
        |entry, _src_dir| {
            if entry
                .absolute_path
                .extension()
                .is_none_or(|ext| ext != "rs")
                || entry
                    .absolute_path
                    .to_str()
                    .is_some_and(|s| s.contains(TEST_DIR_FRAGMENT))
                || !entry.absolute_path.exists()
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let lines: Vec<&str> = content.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if dead_code_pattern.is_match(line) {
                    let item_name = find_dead_code_item(
                        &lines,
                        i,
                        &struct_pattern,
                        &fn_pattern,
                        &field_pattern,
                    )
                    .unwrap_or_else(|| "allow(dead_code)".to_owned());
                    violations.push(QualityViolation::DeadCodeAllowNotPermitted {
                        file: entry.absolute_path.clone(),
                        line: i + 1,
                        item_name,
                        severity: Severity::Warning,
                    });
                }
            }

            Ok(())
        },
    )?;

    Ok(violations)
}

fn find_dead_code_item(
    lines: &[&str],
    start_idx: usize,
    struct_pattern: &Regex,
    fn_pattern: &Regex,
    field_pattern: &Regex,
) -> Option<String> {
    let end = std::cmp::min(start_idx + 5, lines.len());
    for line in lines.iter().take(end).skip(start_idx) {
        if let Some(name) = struct_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("struct {}", name.as_str()));
        }

        if let Some(name) = fn_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("fn {}", name.as_str()));
        }

        if let Some(name) = field_pattern.captures(line).and_then(|c| c.get(1)) {
            return Some(format!("field {}", name.as_str()));
        }
    }

    None
}
