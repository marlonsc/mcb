use std::path::Path;

use super::super::utils::get_suffix;
use super::super::violation::NamingViolation;
use crate::traits::violation::Severity;

pub fn validate_file_suffix(
    path: &Path,
    crate_name: &str,
    server_crate: &str,
    domain_crate: &str,
) -> Option<NamingViolation> {
    let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let path_str = path.to_str()?;

    // Skip standard files
    if file_name == "lib" || file_name == "mod" || file_name == "main" || file_name == "build" {
        return None;
    }

    // Check repository files should have _repository suffix
    if (path_str.contains("/repositories/") || path_str.contains("/adapters/repository/"))
        && !file_name.ends_with("_repository")
        && file_name != "mod"
    {
        return Some(NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Repository".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: "_repository".to_owned(),
            severity: Severity::Warning,
        });
    }

    // Check handler files in server crate
    if crate_name == server_crate && path_str.contains("/handlers/") {
        // Handlers should have descriptive names (snake_case tool names)
        // but NOT have _handler suffix (that's redundant with directory)
        if file_name.ends_with("_handler") {
            return Some(NamingViolation::BadFileSuffix {
                path: path.to_path_buf(),
                component_type: "Handler".to_owned(),
                current_suffix: "_handler".to_owned(),
                expected_suffix: "<tool_name> (no _handler suffix in handlers/ dir)".to_owned(),
                severity: Severity::Info,
            });
        }
    }

    // Check service files should have _service suffix if in services directory
    // Note: mcb-domain/domain_services contains interfaces, not implementations
    // so we skip suffix validation for that directory
    if path_str.contains("/services/")
        && !path_str.contains("/domain_services/")
        && crate_name != domain_crate
        && !file_name.ends_with("_service")
        && file_name != "mod"
    {
        return Some(NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Service".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: "_service".to_owned(),
            severity: Severity::Info,
        });
    }

    // Check factory files - allow both 'factory.rs' and '*_factory.rs'
    // A file named exactly "factory.rs" is valid (e.g., provider_factory module)
    if file_name.contains("factory") && !file_name.ends_with("_factory") && file_name != "factory" {
        return Some(NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Factory".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: "_factory".to_owned(),
            severity: Severity::Info,
        });
    }

    None
}
