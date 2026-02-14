use super::violation::OrganizationViolation;
use crate::scan::for_each_crate_rs_path;
use crate::{Result, Severity, ValidationConfig};

/// Verifies that files are located in the correct directories based on their architectural role.
pub fn validate_file_placement(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    for_each_crate_rs_path(config, |path, src_dir, crate_name| {
        let rel_path = path.strip_prefix(src_dir).ok();
        let Some(path_str) = path.to_str() else {
            return Ok(());
        };
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check for adapter implementations in domain crate
        if crate_name.contains("domain") && path_str.contains("/adapters/") {
            violations.push(OrganizationViolation::FileInWrongLocation {
                file: path.to_path_buf(),
                current_location: "domain/adapters".to_string(),
                expected_location: "infrastructure/adapters".to_string(),
                reason: "Adapters belong in infrastructure layer".to_string(),
                severity: Severity::Error,
            });
        }

        // Check for port definitions in infrastructure
        if crate_name.contains("infrastructure") && path_str.contains("/ports/") {
            violations.push(OrganizationViolation::FileInWrongLocation {
                file: path.to_path_buf(),
                current_location: "infrastructure/ports".to_string(),
                expected_location: "domain/ports".to_string(),
                reason: "Ports (interfaces) belong in domain layer".to_string(),
                severity: Severity::Error,
            });
        }

        // Check for config files outside config directories
        // Exclude handler files (e.g., config_handlers.rs) - these are HTTP handlers, not config files
        if file_name.contains("config")
            && !file_name.contains("handler")
            && !path_str.contains("/config/")
            && !path_str.contains("/config.rs")
            && !path_str.contains("/admin/")
        // Admin config handlers are valid
        {
            // Allow config.rs at root level
            if rel_path.is_some_and(|p| p.components().count() > 1) {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "scattered".to_string(),
                    expected_location: "config/ directory".to_string(),
                    reason: "Configuration should be centralized".to_string(),
                    severity: Severity::Info,
                });
            }
        }

        // Check for error handling spread across modules
        if file_name == "error.rs" {
            // Check that it's at the crate root or in a designated error module
            if rel_path.is_some_and(|p| {
                let depth = p.components().count();
                depth > 2 && !path_str.contains("/error/")
            }) {
                violations.push(OrganizationViolation::FileInWrongLocation {
                    file: path.to_path_buf(),
                    current_location: "nested error.rs".to_string(),
                    expected_location: "crate root or error/ module".to_string(),
                    reason: "Error types should be centralized".to_string(),
                    severity: Severity::Info,
                });
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
