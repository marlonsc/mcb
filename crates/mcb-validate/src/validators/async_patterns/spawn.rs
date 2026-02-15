use crate::filters::LanguageId;
use crate::pattern_registry::required_pattern;
use crate::scan::for_each_scan_file;
use crate::{Result, Severity, ValidationConfig};

use super::violation::AsyncViolation;

/// Detect spawn without await patterns
pub fn validate_spawn_patterns(config: &ValidationConfig) -> Result<Vec<AsyncViolation>> {
    let mut violations = Vec::new();

    // Pattern: tokio::spawn without assigning to variable or awaiting
    let spawn_pattern = required_pattern("ASYNC001.tokio_spawn")?;
    let assigned_spawn_pattern = required_pattern("ASYNC001.assigned_spawn")?;
    let fn_pattern = required_pattern("ASYNC001.fn_decl")?;

    // Function name patterns that indicate intentional fire-and-forget spawns
    // Includes constructor patterns that often spawn background workers
    let background_fn_patterns = [
        "spawn",
        "background",
        "graceful",
        "shutdown",
        "start",
        "run",
        "worker",
        "daemon",
        "listener",
        "handler",
        "process",
        "new",
        "with_",
        "init",
        "create",
        "build", // Constructor patterns
    ];

    for_each_scan_file(config, Some(LanguageId::Rust), false, |entry, _src_dir| {
        let path = &entry.absolute_path;
        if path.to_str().is_some_and(|s| s.contains("/tests/")) {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;
        let mut in_test_module = false;
        let mut current_fn_name = String::new();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Track test modules
            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                continue;
            }

            if in_test_module {
                continue;
            }

            // Track current function name
            if let Some(cap) = fn_pattern.captures(line) {
                current_fn_name = cap.get(1).map_or("", |m| m.as_str()).to_lowercase();
            }

            // Check for unassigned spawn
            if spawn_pattern.is_match(line) && !assigned_spawn_pattern.is_match(line) {
                // Check if it's being used in a chain (e.g., .await)
                if !line.contains(".await") && !line.contains("let _") {
                    // Skip if function name suggests fire-and-forget is intentional
                    let is_background_fn = background_fn_patterns
                        .iter()
                        .any(|p| current_fn_name.contains(p));
                    if is_background_fn {
                        continue;
                    }
                    violations.push(AsyncViolation::UnawaitedSpawn {
                        file: path.clone(),
                        line: line_num + 1,
                        context: trimmed.chars().take(80).collect(),
                        severity: Severity::Info,
                    });
                }
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
