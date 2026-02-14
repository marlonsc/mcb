use regex::Regex;

use super::{QualityValidator, QualityViolation};
use crate::scan::for_each_scan_rs_path;
use crate::{Result, Severity};

/// Scans for and reports usage of `allow(dead_code)` attributes.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();
    // # Code Quality Violation (QUAL001)
    // Static regex initialization using .unwrap() is risky in production.
    //
    // TODO(QUAL001): Use LazyLock or proper error handling for Regex creation.
    let dead_code_pattern = Regex::new(r"#\[allow\([^\)]*dead_code[^\)]*\)\]").unwrap();
    let struct_pattern = Regex::new(r"pub\s+struct\s+(\w+)").unwrap();
    let fn_pattern = Regex::new(r"(?:pub\s+)?fn\s+(\w+)").unwrap();
    let field_pattern = Regex::new(r"(?:pub\s+)?(\w+):\s+").unwrap();

    for_each_scan_rs_path(&validator.config, false, |path, _src_dir| {
        if path.extension().is_none_or(|ext| ext != "rs")
            || path.to_string_lossy().contains("/tests/")
            || !path.exists()
        {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if dead_code_pattern.is_match(line) {
                let item_name =
                    find_dead_code_item(&lines, i, &struct_pattern, &fn_pattern, &field_pattern)
                        .unwrap_or_else(|| "allow(dead_code)".to_string());
                violations.push(QualityViolation::DeadCodeAllowNotPermitted {
                    file: path.to_path_buf(),
                    line: i + 1,
                    item_name,
                    severity: Severity::Warning,
                });
            }
        }

        Ok(())
    })?;

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
