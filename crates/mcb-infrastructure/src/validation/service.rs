//!
//! **Documentation**: [docs/modules/infrastructure.md](../../../../docs/modules/infrastructure.md)
//!
//! Validation Service Implementation
//!
//! # Overview
//! The `InfraValidationService` adapts the `mcb-validate` toolkit into the domain's
//! `ValidationServiceInterface` port. It serves as the bridge between the core domain's
//! need for quality assurance and the infrastructure-level tools that perform analysis.
//!
//! # Responsibilities
//! - **Workspace Validation**: Running suite of validators against the entire project.
//! - **File Analysis**: Targeted validation and complexity analysis for individual files.
//! - **Rule Discovery**: Exposing available validation rules and their metadata.
//! - **Complexity Metrics**: Calculating cyclomatic and cognitive complexity scores.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};

/// Infrastructure validation service using mcb-validate.
///
/// A stateless adapter that orchestrates the `mcb-validate` library to perform
/// architectural compliance checks, code quality analysis, and rule enforcement.
pub struct InfraValidationService;

impl InfraValidationService {
    /// Create a new validation service
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl Default for InfraValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for InfraValidationService {
    async fn validate(
        &self,
        workspace_root: &Path,
        validators: Option<&[String]>,
        severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        run_validation(workspace_root, validators, severity_filter)
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        let mut validators = mcb_domain::registry::validation::list_validator_names();
        validators.sort_unstable();
        Ok(validators)
    }

    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        run_file_validation(file_path, validators)
    }

    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>> {
        let entries = mcb_domain::registry::validation::list_validator_entries();
        let mut rules: Vec<RuleInfo> = entries
            .iter()
            .filter(|(name, _)| category.is_none_or(|c| *name == c))
            .map(|(name, description)| RuleInfo {
                id: (*name).to_owned(),
                category: (*name).to_owned(),
                severity: "warning".to_owned(),
                description: (*description).to_owned(),
                engine: "linkme".to_owned(),
            })
            .collect();
        rules.sort_unstable_by(|a, b| a.id.cmp(&b.id));
        Ok(rules)
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        include_functions: bool,
    ) -> Result<ComplexityReport> {
        analyze_file_complexity(file_path, include_functions)
    }
}

fn run_validation(
    workspace_root: &Path,
    validators: Option<&[String]>,
    severity_filter: Option<&str>,
) -> Result<ValidationReport> {
    let root = workspace_root.to_path_buf();
    let validators_list = if let Some(names) = validators {
        mcb_domain::registry::validation::build_named_validators(&root, names)?
    } else {
        mcb_domain::registry::validation::build_all_validators(&root)?
    };

    let config = mcb_domain::ports::validation::ValidationConfig::new(&root);

    let mut all_violations: Vec<Box<dyn mcb_domain::ports::validation::Violation>> = Vec::new();
    for v in &validators_list {
        match v.validate(&config) {
            Ok(violations) => all_violations.extend(violations),
            Err(e) => {
                return Err(mcb_domain::error::Error::internal(format!(
                    "validator '{}' failed: {}",
                    v.name(),
                    e
                )));
            }
        }
    }

    Ok(mcb_domain::registry::validation::build_report(
        &all_violations,
        severity_filter,
    ))
}

/// Traverse parent directories to find the workspace root (directory containing Cargo.toml).
fn find_workspace_root_from(start: &Path) -> Option<std::path::PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?
    } else {
        start
    };
    loop {
        if current.join("Cargo.toml").exists() {
            return Some(current.to_path_buf());
        }
        current = current.parent()?;
    }
}

fn run_file_validation(
    file_path: &Path,
    validators: Option<&[String]>,
) -> Result<ValidationReport> {
    // For single file validation, we need to find the workspace root
    // and run validation scoped to that file
    let workspace_root = find_workspace_root_from(file_path)
        .unwrap_or_else(|| file_path.parent().unwrap_or(file_path).to_path_buf());

    // Run standard validation - mcb-validate doesn't have single-file mode yet
    // So we run full validation and filter to the specific file
    let full_report = run_validation(&workspace_root, validators, None)?;

    let file_str = file_path.to_str().unwrap_or_default().to_owned();
    let file_violations: Vec<ViolationEntry> = full_report
        .violations
        .into_iter()
        .filter(|v| v.file.as_ref().is_some_and(|f| f.contains(&file_str)))
        .collect();

    let total = file_violations.len();
    let errors = file_violations
        .iter()
        .filter(|v| v.severity == "ERROR")
        .count();
    let warnings = file_violations
        .iter()
        .filter(|v| v.severity == "WARNING")
        .count();
    let infos = total.saturating_sub(errors).saturating_sub(warnings);
    Ok(ValidationReport {
        total_violations: total,
        errors,
        warnings,
        infos,
        passed: errors == 0,
        violations: file_violations,
    })
}
fn analyze_file_complexity(file_path: &Path, include_functions: bool) -> Result<ComplexityReport> {
    let content = std::fs::read_to_string(file_path).map_err(|e| {
        mcb_domain::error::Error::io_with_source(
            format!("failed to read {}", file_path.display()),
            e,
        )
    })?;

    let sloc = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && !trimmed.starts_with("//")
        })
        .count();

    let files = vec![(file_path.to_path_buf(), content)];
    let functions = mcb_domain::utils::analysis::collect_functions(&files)?;

    let total_cyclomatic: f64 = functions.iter().map(|f| f64::from(f.complexity)).sum();
    let fn_count = functions.len().max(1) as f64;
    let avg_cyclomatic = total_cyclomatic / fn_count;
    let cognitive = avg_cyclomatic * 1.2;

    // Simplified Maintainability Index (Microsoft variant)
    let mi = if sloc > 0 {
        let raw = 171.0
            - 5.2 * avg_cyclomatic.max(1.0).ln()
            - 0.23 * avg_cyclomatic
            - 16.2 * (sloc as f64).ln();
        (raw * 100.0 / 171.0).clamp(0.0, 100.0)
    } else {
        100.0
    };

    let function_metrics = if include_functions {
        functions
            .iter()
            .map(|f| FunctionComplexity {
                name: f.name.clone(),
                line: f.line,
                cyclomatic: f64::from(f.complexity),
                cognitive: f64::from(f.complexity) * 1.2,
                sloc: 0,
            })
            .collect()
    } else {
        vec![]
    };

    Ok(ComplexityReport {
        file: file_path.display().to_string(),
        cyclomatic: avg_cyclomatic,
        cognitive,
        maintainability_index: mi,
        sloc,
        functions: function_metrics,
    })
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::services::{
    SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry, VALIDATION_SERVICE_NAME,
};

#[allow(unsafe_code)]
#[linkme::distributed_slice(SERVICES_REGISTRY)]
static VALIDATION_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: VALIDATION_SERVICE_NAME,
    build: ServiceBuilder::Validation(|_context| {
        Ok(std::sync::Arc::new(InfraValidationService::new()))
    }),
};
