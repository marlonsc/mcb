use super::violation::OrganizationViolation;
use crate::scan::{for_each_scan_rs_path, is_test_path};
use crate::{Result, Severity, ValidationConfig};
use regex::Regex;
use std::sync::OnceLock;

static PORT_TRAIT_PATTERN: OnceLock<Regex> = OnceLock::new();
static HANDLER_STRUCT_PATTERN: OnceLock<Regex> = OnceLock::new();
static ADAPTER_IMPL_PATTERN: OnceLock<Regex> = OnceLock::new();

/// Enforces strict directory placement rules for specific component types (ports, adapters, handlers).
///
/// Validates that:
/// - Port traits are located in `domain/ports/`.
/// - Adapter implementations are located in `infrastructure/adapters/`.
/// - Handlers are located in `server/handlers/`.
pub fn validate_strict_directory(config: &ValidationConfig) -> Result<Vec<OrganizationViolation>> {
    let mut violations = Vec::new();

    // Patterns for detecting component types
    let port_trait_pattern = PORT_TRAIT_PATTERN.get_or_init(|| {
        Regex::new(
            r"(?:pub\s+)?trait\s+([A-Z][a-zA-Z0-9_]*(?:Provider|Service|Repository|Interface))\s*:",
        )
        .expect("Invalid port regex")
    });
    let handler_struct_pattern = HANDLER_STRUCT_PATTERN.get_or_init(|| {
        Regex::new(r"(?:pub\s+)?struct\s+([A-Z][a-zA-Z0-9_]*Handler)")
            .expect("Invalid handler regex")
    });
    let adapter_impl_pattern = ADAPTER_IMPL_PATTERN.get_or_init(|| {
        Regex::new(r"impl\s+(?:async\s+)?([A-Z][a-zA-Z0-9_]*(?:Provider|Repository))\s+for\s+([A-Z][a-zA-Z0-9_]*)").expect("Invalid adapter regex")
    });

    for_each_scan_rs_path(config, true, |path, src_dir| {
        let is_domain_crate = src_dir.to_string_lossy().contains("domain");
        let is_infrastructure_crate = src_dir.to_string_lossy().contains("infrastructure");
        let is_server_crate = src_dir.to_string_lossy().contains("server");

        let path_str = path.to_string_lossy();

        // Skip test files
        if is_test_path(&path_str) {
            return Ok(());
        }

        // Skip mod.rs and lib.rs (aggregator files)
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if file_name == "mod.rs" || file_name == "lib.rs" {
            return Ok(());
        }

        let content = std::fs::read_to_string(path)?;

        // Check for port traits outside allowed directories
        if is_domain_crate {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(cap) = port_trait_pattern.captures(line) {
                    let trait_name = cap.get(1).map_or("", |m| m.as_str());

                    // Allowed in: ports/, domain_services/, repositories/
                    // Domain service interfaces belong in domain_services
                    // Repository interfaces belong in repositories
                    let in_allowed_dir = path_str.contains("/ports/")
                        || path_str.contains("/domain_services/")
                        || path_str.contains("/repositories/");

                    if !in_allowed_dir {
                        violations.push(OrganizationViolation::PortOutsidePorts {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            trait_name: trait_name.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        // Check for handlers outside allowed directories
        if is_server_crate {
            for (line_num, line) in content.lines().enumerate() {
                if let Some(cap) = handler_struct_pattern.captures(line) {
                    let handler_name = cap.get(1).map_or("", |m| m.as_str());

                    // Allowed in: handlers/, admin/, tools/, and cross-cutting files
                    // Admin handlers belong in admin/
                    // Tool handlers belong in tools/
                    // Auth handlers are cross-cutting concerns
                    let in_allowed_location = path_str.contains("/handlers/")
                        || path_str.contains("/admin/")
                        || path_str.contains("/tools/")
                        || file_name == "auth.rs"
                        || file_name == "middleware.rs";

                    if !in_allowed_location {
                        violations.push(OrganizationViolation::HandlerOutsideHandlers {
                            file: path.to_path_buf(),
                            line: line_num + 1,
                            handler_name: handler_name.to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        // Check for adapter implementations outside allowed directories
        if is_infrastructure_crate {
            for line in content.lines() {
                if let Some(cap) = adapter_impl_pattern.captures(line) {
                    let _trait_name = cap.get(1).map_or("", |m| m.as_str());
                    let _impl_name = cap.get(2).map_or("", |m| m.as_str());

                    // Allowed in: adapters/, di/, and cross-cutting concern directories
                    // crypto/, cache/, health/, events/ are infrastructure cross-cutting concerns
                    let in_allowed_dir = path_str.contains("/adapters/")
                        || path_str.contains("/di/")
                        || path_str.contains("/crypto/")
                        || path_str.contains("/cache/")
                        || path_str.contains("/health/")
                        || path_str.contains("/events/")
                        || path_str.contains("/sync/")
                        || path_str.contains("/config/")
                        || path_str.contains("/infrastructure/") // Null impls for DI
                        || file_name.contains("factory")
                        || file_name.contains("bootstrap");

                    if !in_allowed_dir {
                        let current_dir = path
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_default();

                        violations.push(OrganizationViolation::StrictDirectoryViolation {
                            file: path.to_path_buf(),
                            component_type: crate::ComponentType::Adapter,
                            current_directory: current_dir,
                            expected_directory: "infrastructure/adapters/".to_string(),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }

        Ok(())
    })?;

    Ok(violations)
}
