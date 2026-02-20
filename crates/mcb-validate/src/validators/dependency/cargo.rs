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

        // Check [dependencies] section
        if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
            for dep_name in deps.keys() {
                if dep_name.starts_with(MCB_DEPENDENCY_PREFIX) && dep_name != crate_name {
                    let dep_crate = dep_name.replace('_', "-");
                    if !allowed.contains(&dep_crate) {
                        violations.push(DependencyViolation::ForbiddenCargoDepedency {
                            crate_name: crate_name.clone(),
                            forbidden_dep: dep_crate,
                            location: cargo_toml.clone(),
                            severity: Severity::Error,
                        });
                    }
                }
            }
        }

        // Check [dev-dependencies] section (more lenient - allow test utilities)
        // Dev dependencies are allowed to have more flexibility
    }

    Ok(violations)
}
