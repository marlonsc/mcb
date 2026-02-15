use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::Severity;
use crate::traits::violation::ViolationCategory;

/// Wrapper for dependency cycle to provide custom formatting.
#[derive(Clone, Serialize, Deserialize)]
pub struct DependencyCycle(pub Vec<String>);

impl std::fmt::Debug for DependencyCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.join(" -> "))
    }
}

define_violations! {
    dynamic_severity,
    ViolationCategory::Architecture,
    pub enum DependencyViolation {
        /// Forbidden dependency in Cargo.toml
        #[violation(
            id = "DEP001",
            severity = Error,
            message = "Forbidden dependency: {crate_name} depends on {forbidden_dep} (in {location})",
            suggestion = "Remove {forbidden_dep} from {crate_name}/Cargo.toml"
        )]
        ForbiddenCargoDepedency {
            crate_name: String,
            forbidden_dep: String,
            location: PathBuf,
            severity: Severity,
        },
        /// Forbidden use statement in source code
        #[violation(
            id = "DEP002",
            severity = Error,
            message = "Forbidden use: {crate_name} uses {forbidden_dep} at {file}:{line} - {context}",
            suggestion = "Access {forbidden_dep} through allowed layer"
        )]
        ForbiddenUseStatement {
            crate_name: String,
            forbidden_dep: String,
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// Circular dependency detected
        #[violation(
            id = "DEP003",
            severity = Error,
            message = "Circular dependency: {cycle:?}",
            suggestion = "Extract shared types to the domain crate"
        )]
        CircularDependency {
            cycle: DependencyCycle,
            severity: Severity,
        },
        /// Admin surface imports repository ports outside approved composition roots.
        #[violation(
            id = "DEP004",
            severity = Error,
            message = "Admin bypass import: {file}:{line} - {context}",
            suggestion = "Route admin operations through ToolHandlers/UnifiedExecution and keep repository imports in approved composition roots only"
        )]
        AdminBypassImport {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
        /// CLI code bypasses unified execution by calling validate crate directly.
        #[violation(
            id = "DEP005",
            severity = Error,
            message = "CLI bypass path: {file}:{line} - {context}",
            suggestion = "Route CLI business commands through the unified execution path instead of direct mcb_validate calls"
        )]
        CliBypassPath {
            file: PathBuf,
            line: usize,
            context: String,
            severity: Severity,
        },
    }
}
