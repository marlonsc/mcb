use std::path::Path;

use super::super::violation::NamingViolation;
use crate::constants::common::STANDARD_SKIP_FILES;
use crate::traits::violation::Severity;
use crate::validators::naming::constants::*;

pub fn validate_ca_naming(
    path: &Path,
    crate_name: &str,
    domain_crate: &str,
    infrastructure_crate: &str,
    server_crate: &str,
) -> Option<NamingViolation> {
    let file_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let path_str = path.to_str()?;

    // Skip standard files
    if STANDARD_SKIP_FILES.contains(&file_name) {
        return None;
    }

    // Domain crate: port traits should be in ports/
    if crate_name == domain_crate {
        // Files with "provider" in name should be in ports/providers/
        if file_name.contains(CA_DOMAIN_PROVIDER_KEYWORD)
            && !path_str.contains(CA_PORTS_PROVIDERS_DIR)
            && !path_str.contains(CA_PORTS_DIR)
        {
            return Some(NamingViolation::BadCaNaming {
                path: path.to_path_buf(),
                detected_type: "Provider Port".to_owned(),
                issue: "Provider file outside ports/ directory".to_owned(),
                suggestion: "Move to ports/providers/".to_owned(),
                severity: Severity::Warning,
            });
        }

        // Files with "repository" in name should be in repositories/
        if file_name.contains(CA_DOMAIN_REPOSITORY_KEYWORD)
            && !path_str.contains(CA_REPOSITORIES_DIR)
            && !path_str.contains(CA_ADAPTERS_REPOSITORY_DIR)
        {
            return Some(NamingViolation::BadCaNaming {
                path: path.to_path_buf(),
                detected_type: "Repository Port".to_owned(),
                issue: "Repository file outside repositories/ directory".to_owned(),
                suggestion: "Move to repositories/".to_owned(),
                severity: Severity::Warning,
            });
        }
    }

    // Infrastructure crate: adapters should be in adapters/
    if crate_name == infrastructure_crate {
        // Implementation files should be in adapters/
        if (file_name.ends_with(CA_INFRA_IMPL_SUFFIX)
            || file_name.contains(CA_INFRA_ADAPTER_KEYWORD))
            && !path_str.contains(CA_ADAPTERS_DIR)
        {
            return Some(NamingViolation::BadCaNaming {
                path: path.to_path_buf(),
                detected_type: "Adapter".to_owned(),
                issue: "Adapter/implementation file outside adapters/ directory".to_owned(),
                suggestion: "Move to adapters/".to_owned(),
                severity: Severity::Warning,
            });
        }

        // DI modules should be in di/
        if file_name.contains(CA_MODULE_KEYWORD) && !path_str.contains(CA_DI_DIR) {
            return Some(NamingViolation::BadCaNaming {
                path: path.to_path_buf(),
                detected_type: "DI Module".to_owned(),
                issue: "Module file outside di/ directory".to_owned(),
                suggestion: "Move to di/modules/".to_owned(),
                severity: Severity::Info,
            });
        }
    }

    // Server crate: handlers should be in handlers/ or admin/
    if crate_name == server_crate {
        // Allow handlers in handlers/, admin/, or tools/ directories
        let in_allowed_handler_dir = CA_HANDLER_DIRS.iter().any(|d| path_str.contains(d));
        if file_name.contains(CA_HANDLER_KEYWORD) && !in_allowed_handler_dir {
            return Some(NamingViolation::BadCaNaming {
                path: path.to_path_buf(),
                detected_type: "Handler".to_owned(),
                issue: "Handler file outside handlers/ directory".to_owned(),
                suggestion: "Move to handlers/, admin/, or tools/".to_owned(),
                severity: Severity::Warning,
            });
        }
    }

    None
}
