use super::violation::OrganizationViolation;
use crate::filters::LanguageId;
use crate::scan::{for_each_scan_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::sync::OnceLock;

static ARC_NEW_SERVICE_PATTERN: OnceLock<Regex> = OnceLock::new();
static SERVER_IMPORT_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Checks for violations of Clean Architecture layer boundaries.
///
/// Detects issues such as:
/// - Server layer code directly instantiating services (bypassing DI).
/// - Application layer code importing from the server layer (dependency inversion violation).
pub fn validate_layer_violations(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Patterns for detecting layer violations
    let arc_new_service_pattern = ARC_NEW_SERVICE_PATTERN.get_or_init(|| {
        Regex::new(r"Arc::new\s*\(\s*([A-Z][a-zA-Z0-9_]*(?:Service|Provider|Repository))::new")
            .expect("Invalid arc_new_service regex")
    });
    let server_import_pattern = SERVER_IMPORT_PATTERN.get_or_init(|| {
        Regex::new(r"use\s+(?:crate::|super::)*server::").expect("Invalid server_import regex")
    });

    for_each_scan_file(config, Some(LanguageId::Rust), true, |entry, _src_dir| {
        let Some(path_str) = entry.absolute_path.to_str() else {
            return Ok(());
        };

        // Skip test files
        if is_test_path(path_str) {
            return Ok(());
        }

        // Determine current layer
        let is_server_layer = path_str.contains("/server/");
        let is_application_layer = path_str.contains("/application/");
        let is_infrastructure_layer = path_str.contains("/infrastructure/");

        let content = std::fs::read_to_string(&entry.absolute_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Track test modules to skip
        let mut in_test_module = false;
        let mut test_brace_depth: i32 = 0;
        let mut brace_depth: i32 = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track test module boundaries
            if trimmed.contains("#[cfg(test)]") {
                in_test_module = true;
                test_brace_depth = brace_depth;
            }

            brace_depth += i32::try_from(line.chars().filter(|c| *c == '{').count()).unwrap_or(0);
            brace_depth -= i32::try_from(line.chars().filter(|c| *c == '}').count()).unwrap_or(0);

            if in_test_module && brace_depth < test_brace_depth {
                in_test_module = false;
            }

            // Skip test modules
            if in_test_module {
                continue;
            }

            // Skip comments
            if trimmed.starts_with("//") {
                continue;
            }

            // Check: Server layer creating services directly
            if is_server_layer && let Some(cap) = arc_new_service_pattern.captures(line) {
                let service_name = cap.get(1).map_or("", |m| m.as_str());

                // Skip if it's in a builder or factory file
                let file_name = entry
                    .absolute_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");
                if file_name.contains("builder")
                    || file_name.contains("factory")
                    || file_name.contains("bootstrap")
                {
                    continue;
                }

                violations.push(OrganizationViolation::ServerCreatingServices {
                    file: entry.absolute_path.clone(),
                    line: line_num + 1,
                    service_name: service_name.to_owned(),
                    suggestion: "Use DI container to resolve services".to_owned(),
                    severity: Severity::Warning,
                });
            }

            // Check: Application layer importing from server
            if (is_application_layer || is_infrastructure_layer)
                && server_import_pattern.is_match(line)
                && !trimmed.contains("pub use")
            {
                violations.push(OrganizationViolation::ApplicationImportsServer {
                    file: entry.absolute_path.clone(),
                    line: line_num + 1,
                    import_statement: trimmed.to_owned(),
                    severity: Severity::Warning,
                });
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
