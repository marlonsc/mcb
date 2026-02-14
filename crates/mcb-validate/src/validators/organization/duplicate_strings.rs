use super::violation::OrganizationViolation;
use crate::scan::{for_each_crate_rs_path, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::OnceLock;

static STRING_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Scans for string literals duplicated across multiple files that should be centralized.
pub fn validate_duplicate_strings(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();
    let mut string_occurrences: HashMap<String, Vec<(PathBuf, usize)>> = HashMap::new();

    // Pattern for string literals (15+ chars to reduce noise)
    let string_pattern = STRING_PATTERN
        .get_or_init(|| Regex::new(r#""([^"\\]{15,})""#).expect("Invalid duplicate strings regex"));

    for_each_crate_rs_path(config, |path, _src_dir, _crate_name| {
        // Skip constants files (they define string constants)
        let file_name = path.file_name().and_then(|n| n.to_str());
        if file_name.is_some_and(|n| n.contains("constant")) {
            return Ok(());
        }

        // Skip test files
        let path_str = path.to_string_lossy();
        if is_test_path(&path_str) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut in_test_module = false;

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and doc strings
            if trimmed.starts_with("//") || trimmed.starts_with("#[") {
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

            // Skip const/static definitions
            if trimmed.starts_with("const ")
                || trimmed.starts_with("pub const ")
                || trimmed.starts_with("static ")
                || trimmed.starts_with("pub static ")
            {
                continue;
            }

            for cap in string_pattern.captures_iter(line) {
                let string_val = cap.get(1).map_or("", |m| m.as_str());

                // Skip common patterns that are OK to repeat
                if string_val.contains("{}")           // Format strings
                    || string_val.starts_with("test_")  // Test names
                    || string_val.starts_with("Error")  // Error messages
                    || string_val.starts_with("error")
                    || string_val.starts_with("Failed")
                    || string_val.starts_with("Invalid")
                    || string_val.starts_with("Cannot")
                    || string_val.starts_with("Unable")
                    || string_val.starts_with("Missing")
                    || string_val.contains("://")       // URLs
                    || string_val.contains(".rs")       // File paths
                    || string_val.contains(".json")
                    || string_val.contains(".toml")
                    || string_val.ends_with("_id")      // ID fields
                    || string_val.ends_with("_key")     // Key fields
                    || string_val.starts_with("pub ")   // Code patterns
                    || string_val.starts_with("fn ")
                    || string_val.starts_with("let ")
                    || string_val.starts_with("CARGO_") // env!() macros
                    || string_val.contains("serde_json")// Code patterns
                    || string_val.contains(".to_string()")
                // Method chains
                {
                    continue;
                }

                string_occurrences
                    .entry(string_val.to_string())
                    .or_default()
                    .push((path.to_path_buf(), line_num + 1));
            }
        }

        Ok(())
    })?;

    // Report strings that appear in 4+ files (higher threshold)
    for (value, occurrences) in string_occurrences {
        let unique_files: HashSet<_> = occurrences.iter().map(|(f, _)| f).collect();
        if unique_files.len() >= 4 {
            violations.push(OrganizationViolation::DuplicateStringLiteral {
                value,
                occurrences,
                suggestion: "Consider creating a named constant".to_string(),
                severity: Severity::Info,
            });
        }
    }

    Ok(violations)
}
