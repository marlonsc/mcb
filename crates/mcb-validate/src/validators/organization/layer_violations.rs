//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md#organization)
//!
use super::constants::{
    APPLICATION_LAYER_PATH, ARC_NEW_SERVICE_REGEX, INFRASTRUCTURE_LAYER_PATH, SERVER_IMPORT_REGEX,
    SERVER_LAYER_PATH, SERVICE_CREATION_BYPASS_FILES,
};
use super::violation::OrganizationViolation;
use crate::constants::common::PUB_USE_PREFIX;
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
        if entry.absolute_path.to_str().is_none_or(is_test_path) {
            return Ok(());
        }

        // Determine current layer
        let is_server_layer = path_str.contains(SERVER_LAYER_PATH);
        let is_application_layer = path_str.contains(APPLICATION_LAYER_PATH);
        let is_infrastructure_layer = path_str.contains(INFRASTRUCTURE_LAYER_PATH);

        let content = std::fs::read_to_string(&entry.absolute_path)?;
        let file_name = entry
            .absolute_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let bypass_server_creation = SERVICE_CREATION_BYPASS_FILES
            .iter()
            .any(|f| file_name.contains(f));

        crate::validators::for_each_non_test_non_comment_line(
            &content,
            |line_num, line, trimmed| {
                if is_server_layer
                    && !bypass_server_creation
                    && let Some(cap) = arc_new_service_pattern.captures(line)
                {
                    let service_name = cap.get(1).map_or("", |m| m.as_str());
                    violations.push(OrganizationViolation::ServerCreatingServices {
                        file: entry.absolute_path.clone(),
                        line: line_num + 1,
                        service_name: service_name.to_owned(),
                        suggestion: "Use DI container to resolve services".to_owned(),
                        severity: Severity::Warning,
                    });
                }

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
            },
        );

        Ok(())
    })?;

    Ok(violations)
}
