//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::super::violation::NamingViolation;
use mcb_domain::ports::validation::Severity;
use mcb_utils::constants::validate::{ARCH_PATH_HANDLERS, ARCH_PATH_SERVICES};
use mcb_utils::constants::validate::{
    FACTORY_FILE_SUFFIX, REPOSITORY_FILE_SUFFIX, SERVICE_FILE_SUFFIX,
};
use mcb_utils::utils::naming::get_suffix;

/// Validates component file-name suffixes by location (repository, handler,
/// service, factory), returning a `BadFileSuffix` violation when a file's suffix
/// does not match the convention for its directory. Standard files yield `None`.
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

    check_repository_suffix(path, file_name, path_str)
        .or_else(|| check_handler_suffix(path, file_name, path_str, crate_name, server_crate))
        .or_else(|| check_service_suffix(path, file_name, path_str, crate_name, domain_crate))
        .or_else(|| check_factory_suffix(path, file_name))
}

/// Repository files must carry the `_repository` suffix.
fn check_repository_suffix(
    path: &Path,
    file_name: &str,
    path_str: &str,
) -> Option<NamingViolation> {
    // Port trait definitions live in `ports/repositories/` and are entity-named
    // (e.g. `agent.rs` defines `AgentRepository`); the `_repository` suffix
    // convention applies to concrete adapter implementations, not port contracts.
    let in_repo_dir = (path_str.contains("/repositories/")
        || path_str.contains("/adapters/repository/"))
        && !path_str.contains("/ports/");
    (in_repo_dir && !file_name.ends_with(REPOSITORY_FILE_SUFFIX) && file_name != "mod").then(|| {
        NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Repository".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: REPOSITORY_FILE_SUFFIX.to_owned(),
            severity: Severity::Warning,
        }
    })
}

/// Handler files in the server crate must not carry the redundant `_handler`
/// suffix (the `handlers/` directory already encodes the role).
fn check_handler_suffix(
    path: &Path,
    file_name: &str,
    path_str: &str,
    crate_name: &str,
    server_crate: &str,
) -> Option<NamingViolation> {
    let in_handlers = crate_name == server_crate && path_str.contains(ARCH_PATH_HANDLERS);
    (in_handlers && file_name.ends_with("_handler")).then(|| NamingViolation::BadFileSuffix {
        path: path.to_path_buf(),
        component_type: "Handler".to_owned(),
        current_suffix: "_handler".to_owned(),
        expected_suffix: "<tool_name> (no _handler suffix in handlers/ dir)".to_owned(),
        severity: Severity::Info,
    })
}

/// Service files outside the domain crate must carry the `_service` suffix,
/// unless their parent directory already encodes the role.
fn check_service_suffix(
    path: &Path,
    file_name: &str,
    path_str: &str,
    crate_name: &str,
    domain_crate: &str,
) -> Option<NamingViolation> {
    if !path_str.contains(ARCH_PATH_SERVICES) {
        return None;
    }
    // mcb-domain/domain_services holds interfaces, not implementations.
    let in_domain_services = path_str.contains("/domain_services/");
    // A `foo_service/` module folder already encodes the role on the directory;
    // its internal files need not repeat the `_service` suffix.
    let parent_is_service_module = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .is_some_and(|dir| dir.ends_with(SERVICE_FILE_SUFFIX));
    let invalid_service_name = !file_name.ends_with(SERVICE_FILE_SUFFIX) && file_name != "mod";

    (!in_domain_services
        && !parent_is_service_module
        && crate_name != domain_crate
        && invalid_service_name)
        .then(|| NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Service".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: SERVICE_FILE_SUFFIX.to_owned(),
            severity: Severity::Info,
        })
}

/// Factory files must carry the `_factory` suffix, except a file named exactly
/// `factory.rs`.
fn check_factory_suffix(path: &Path, file_name: &str) -> Option<NamingViolation> {
    (file_name.contains("factory")
        && !file_name.ends_with(FACTORY_FILE_SUFFIX)
        && file_name != "factory")
        .then(|| NamingViolation::BadFileSuffix {
            path: path.to_path_buf(),
            component_type: "Factory".to_owned(),
            current_suffix: get_suffix(file_name).to_owned(),
            expected_suffix: FACTORY_FILE_SUFFIX.to_owned(),
            severity: Severity::Info,
        })
}
