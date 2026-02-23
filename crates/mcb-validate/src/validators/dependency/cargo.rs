//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
use crate::constants::common::MCB_DEPENDENCY_PREFIX;
use crate::linters::constants::CARGO_TOML_FILENAME;
use crate::{Result, Severity};

use super::DependencyValidator;
use super::violation::DependencyViolation;

/// Validate Cargo.toml dependencies match Clean Architecture rules
pub fn validate_cargo_dependencies(
    validator: &DependencyValidator,
) -> Result<Vec<DependencyViolation>> {
    let mut violations = Vec::new();

    for (crate_name, allowed) in &validator.allowed_deps {
        let cargo_toml = validator
            .config
            .workspace_root
            .join("crates")
            .join(crate_name)
            .join(CARGO_TOML_FILENAME);

        if !cargo_toml.exists() {
            continue;
        }

        let content = std::fs::read_to_string(&cargo_toml)?;
        let parsed: toml::Value = toml::from_str(&content)?;

        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            violations.extend(
                deps.keys()
                    .filter(|dep_name| {
                        dep_name.starts_with(MCB_DEPENDENCY_PREFIX) && *dep_name != crate_name
                    })
                    .map(|dep_name| dep_name.replace('_', "-"))
                    .filter(|dep_crate| !allowed.contains(dep_crate))
                    .map(|dep_crate| DependencyViolation::ForbiddenCargoDepedency {
                        crate_name: crate_name.clone(),
                        forbidden_dep: dep_crate,
                        location: cargo_toml.clone(),
                        severity: Severity::Error,
                    }),
            );
        }
    }

    Ok(violations)
}
