use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::scan::{for_each_crate_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::sync::OnceLock;

static MAGIC_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Scans for numeric literals that should be extracted as named constants.
pub fn validate_magic_numbers(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Pattern for numeric literals: 5+ digits (skip 4-digit numbers to reduce noise)
    let magic_pattern = MAGIC_PATTERN
        .get_or_init(|| Regex::new(r"\b(\d{5,})\b").expect("Invalid magic number regex"));

    // Allowed patterns (common safe numbers, powers of 2, well-known values, etc.)
    let allowed = [
        // Powers of 2
        "16384",
        "32768",
        "65535",
        "65536",
        "131072",
        "262144",
        "524288",
        "1048576",
        "2097152",
        "4194304",
        // Common memory sizes (in bytes)
        "100000",
        "1000000",
        "10000000",
        "100000000",
        // Time values (seconds)
        "86400",
        "604800",
        "2592000",
        "31536000",
        // Large round numbers (often limits)
        "100000",
        "1000000",
    ];

    for_each_crate_file(
        config,
        Some(LanguageId::Rust),
        |entry, _src_dir, _crate_name| {
            let path = &entry.absolute_path;
            // Skip constants.rs files (they're allowed to have numbers)
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name.is_some_and(|n| n.contains("constant") || n.contains("config")) {
                return Ok(());
            }

            // Skip test files
            let Some(path_str) = path.to_str() else {
                return Ok(());
            };
            if is_test_path(path_str) {
                return Ok(());
            }

            let content = std::fs::read_to_string(path)?;
            let mut in_test_module = false;

            for (line_num, line) in content.lines().enumerate() {
                let trimmed = line.trim();

                // Skip comments
                if trimmed.starts_with("//") {
                    continue;
                }

                // Track test module context
                if trimmed.contains("#[cfg(test)]") {
                    in_test_module = true;
                    continue;
                }

                // Skip test modules
                if in_test_module {
                    continue;
                }

                // Skip const/static definitions (they're creating constants)
                if trimmed.starts_with("const ")
                    || trimmed.starts_with("pub const ")
                    || trimmed.starts_with("static ")
                    || trimmed.starts_with("pub static ")
                {
                    continue;
                }

                // Skip attribute macros (derive, cfg, etc.)
                if trimmed.starts_with("#[") {
                    continue;
                }

                // Skip doc comments
                if trimmed.starts_with("///") || trimmed.starts_with("//!") {
                    continue;
                }

                // Skip assert macros (often use expected values)
                if trimmed.contains("assert") {
                    continue;
                }

                for cap in magic_pattern.captures_iter(line) {
                    let num = cap.get(1).map_or("", |m| m.as_str());

                    // Skip allowed numbers
                    if allowed.contains(&num) {
                        continue;
                    }

                    // Skip numbers that are clearly part of a constant reference
                    // e.g., _1024, SIZE_16384
                    if line.contains(&format!("_{num}")) || line.contains(&format!("{num}_")) {
                        continue;
                    }

                    // Skip underscored numbers (100_000) - they're usually constants
                    if line.contains(&format!(
                        "{}_{}",
                        &num[..num.len().min(3)],
                        &num[num.len().min(3)..]
                    )) {
                        continue;
                    }

                    violations.push(OrganizationViolation::MagicNumber {
                        file: path.clone(),
                        line: line_num + 1,
                        value: num.to_owned(),
                        context: trimmed.to_owned(),
                        suggestion: "Consider using a named constant".to_owned(),
                        severity: Severity::Info,
                    });
                }
            }

            Ok(())
        },
    )?;

    Ok(violations)
}
