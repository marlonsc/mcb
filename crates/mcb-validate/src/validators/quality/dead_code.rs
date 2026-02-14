use std::sync::LazyLock;

use regex::Regex;

use super::{QualityValidator, QualityViolation};
use crate::filters::LanguageId;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity};

static DEAD_CODE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"#\[allow\([^\)]*dead_code[^\)]*\)\]").expect("valid regex literal")
});
static STRUCT_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"pub\s+struct\s+(\w+)").expect("valid regex literal"));
static FN_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?fn\s+(\w+)").expect("valid regex literal"));
static FIELD_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:pub\s+)?(\w+):\s+").expect("valid regex literal"));

/// Scans for and reports usage of `allow(dead_code)` attributes.
pub fn validate(validator: &QualityValidator) -> Result<Vec<QualityViolation>> {
    let mut violations = Vec::new();

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
                    .is_some_and(|s| s.contains("/tests/"))
                || !entry.absolute_path.exists()
            {
                return Ok(());
            }

            let content = std::fs::read_to_string(&entry.absolute_path)?;
            let lines: Vec<&str> = content.lines().collect();

            for (i, line) in lines.iter().enumerate() {
                if DEAD_CODE_PATTERN.is_match(line) {
                    let item_name = find_dead_code_item(
                        &lines,
                        i,
                        &STRUCT_PATTERN,
                        &FN_PATTERN,
                        &FIELD_PATTERN,
                    )
                    .unwrap_or_else(|| "allow(dead_code)".to_string());
                    violations.push(QualityViolation::DeadCodeAllowNotPermitted {
                        file: entry.absolute_path.to_path_buf(),
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
