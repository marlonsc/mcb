//!
//! **Documentation**: [docs/modules/validate.md](../../../../../../docs/modules/validate.md)
//!
use std::path::Path;

use super::super::violation::NamingViolation;
use crate::apply_ca_rule;
use crate::constants::common::STANDARD_SKIP_FILES;
use crate::traits::violation::Severity;
use crate::validators::naming::constants::{
    CA_ADAPTERS_DIR, CA_ADAPTERS_REPOSITORY_DIR, CA_DI_DIR, CA_DOMAIN_PROVIDER_KEYWORD,
    CA_DOMAIN_REPOSITORY_KEYWORD, CA_HANDLER_DIRS, CA_HANDLER_KEYWORD, CA_INFRA_ADAPTER_KEYWORD,
    CA_INFRA_IMPL_SUFFIX, CA_MODULE_KEYWORD, CA_PORTS_DIR, CA_PORTS_PROVIDERS_DIR,
    CA_REPOSITORIES_DIR,
};

fn ca_violation(
    path: &Path,
    detected_type: &str,
    issue: &str,
    suggestion: &str,
    severity: Severity,
) -> NamingViolation {
    NamingViolation::BadCaNaming {
        path: path.to_path_buf(),
        detected_type: detected_type.to_owned(),
        issue: issue.to_owned(),
        suggestion: suggestion.to_owned(),
        severity,
    }
}

#[derive(Clone, Copy)]
enum NameMatch<'a> {
    Contains(&'a str),
    EndsWith(&'a str),
}

fn name_matches(file_name: &str, matcher: NameMatch<'_>) -> bool {
    match matcher {
        NameMatch::Contains(value) => file_name.contains(value),
        NameMatch::EndsWith(value) => file_name.ends_with(value),
    }
}

fn in_any_dir(path_str: &str, dirs: &[&str]) -> bool {
    dirs.iter().any(|dir| path_str.contains(dir))
}

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

    if crate_name == domain_crate {
        return apply_ca_rule!(
            path,
            file_name,
            path_str,
            NameMatch::Contains(CA_DOMAIN_PROVIDER_KEYWORD),
            &[CA_PORTS_PROVIDERS_DIR, CA_PORTS_DIR],
            "Provider Port",
            "Provider file outside ports/ directory",
            "Move to ports/providers/",
            Severity::Warning
        )
        .or_else(|| {
            apply_ca_rule!(
                path,
                file_name,
                path_str,
                NameMatch::Contains(CA_DOMAIN_REPOSITORY_KEYWORD),
                &[CA_REPOSITORIES_DIR, CA_ADAPTERS_REPOSITORY_DIR],
                "Repository Port",
                "Repository file outside repositories/ directory",
                "Move to repositories/",
                Severity::Warning
            )
        });
    }

    if crate_name == infrastructure_crate {
        return apply_ca_rule!(
            path,
            file_name,
            path_str,
            NameMatch::EndsWith(CA_INFRA_IMPL_SUFFIX),
            &[CA_ADAPTERS_DIR],
            "Adapter",
            "Adapter/implementation file outside adapters/ directory",
            "Move to adapters/",
            Severity::Warning
        )
        .or_else(|| {
            apply_ca_rule!(
                path,
                file_name,
                path_str,
                NameMatch::Contains(CA_INFRA_ADAPTER_KEYWORD),
                &[CA_ADAPTERS_DIR],
                "Adapter",
                "Adapter/implementation file outside adapters/ directory",
                "Move to adapters/",
                Severity::Warning
            )
        })
        .or_else(|| {
            apply_ca_rule!(
                path,
                file_name,
                path_str,
                NameMatch::Contains(CA_MODULE_KEYWORD),
                &[CA_DI_DIR],
                "DI Module",
                "Module file outside di/ directory",
                "Move to di/modules/",
                Severity::Info
            )
        });
    }

    if crate_name == server_crate {
        return apply_ca_rule!(
            path,
            file_name,
            path_str,
            NameMatch::Contains(CA_HANDLER_KEYWORD),
            CA_HANDLER_DIRS,
            "Handler",
            "Handler file outside handlers/ directory",
            "Move to handlers/, admin/, or tools/",
            Severity::Warning
        );
    }

    None
}
