use super::constants::{
    APPLICATION_LAYER_PATH, ARC_NEW_SERVICE_REGEX, INFRASTRUCTURE_LAYER_PATH, SERVER_IMPORT_REGEX,
    SERVER_LAYER_PATH, SERVICE_CREATION_BYPASS_FILES,
};
use super::violation::OrganizationViolation;
use crate::constants::common::{CFG_TEST_MARKER, COMMENT_PREFIX, PUB_USE_PREFIX};
use crate::filters::LanguageId;
use crate::pattern_registry::compile_regex;
use crate::scan::{for_each_scan_file, is_test_path};
use crate::{Result, Severity, ValidationConfig};

/// Checks for violations of Clean Architecture layer boundaries.
///
/// Detects issues such as:
/// - Server layer code directly instantiating services (bypassing DI).
/// - Application layer code importing from the server layer (dependency inversion violation).
///
/// # Errors
///
/// Returns an error if file scanning or reading fails.
pub fn validate_layer_violations(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Patterns for detecting layer violations
    let arc_new_service_pattern = compile_regex(ARC_NEW_SERVICE_REGEX)?;
    let server_import_pattern = compile_regex(SERVER_IMPORT_REGEX)?;

    for_each_scan_file(config, Some(LanguageId::Rust), true, |entry, _src_dir| {
        let Some(path_str) = entry.absolute_path.to_str() else {
            return Ok(());
        };

        // Skip test files
        if is_test_path(path_str) {
            return Ok(());
        }

        // Determine current layer
        let is_server_layer = path_str.contains(SERVER_LAYER_PATH);
        let is_application_layer = path_str.contains(APPLICATION_LAYER_PATH);
        let is_infrastructure_layer = path_str.contains(INFRASTRUCTURE_LAYER_PATH);

        let content = std::fs::read_to_string(&entry.absolute_path)?;
        let lines: Vec<&str> = content.lines().collect();

        // Track test modules to skip
        let mut in_test_module = false;
        let mut test_brace_depth: i32 = 0;
        let mut brace_depth: i32 = 0;

        for (line_num, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track test module boundaries
            if trimmed.contains(CFG_TEST_MARKER) {
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
            if trimmed.starts_with(COMMENT_PREFIX) {
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
                if SERVICE_CREATION_BYPASS_FILES
                    .iter()
                    .any(|f| file_name.contains(f))
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
                && !trimmed.contains(PUB_USE_PREFIX)
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
