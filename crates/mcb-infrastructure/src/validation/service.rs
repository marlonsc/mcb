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
        Ok(mcb_domain::registry::validation::list_validator_names())
    }

    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        run_file_validation(file_path, validators)
    }

    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>> {
        Ok(mcb_validate::utils::yaml::get_validation_rules(category))
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
    use mcb_domain::ports::RuleValidatorRequest;
    use mcb_domain::registry::validation::{build_validators, run_validators};

    let root = workspace_root.to_path_buf();
    let validators_list = build_validators(root.clone())?;
    let request = RuleValidatorRequest {
        workspace_root: root,
        validator_names: validators.map(|s| s.to_vec()),
        severity_filter: severity_filter.map(String::from),
        exclude_patterns: None,
    };
    run_validators(&validators_list, &request)
}

fn run_file_validation(
    file_path: &Path,
    validators: Option<&[String]>,
) -> Result<ValidationReport> {
    // For single file validation, we need to find the workspace root
    // and run validation scoped to that file
    let workspace_root = mcb_validate::find_workspace_root_from(file_path)
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

    Ok(mcb_validate::utils::validation_report::from_violations(
        file_violations,
    ))
}

fn analyze_file_complexity(file_path: &Path, include_functions: bool) -> Result<ComplexityReport> {
    use mcb_validate::RcaAnalyzer;

    let analyzer = RcaAnalyzer::new();

    // Get aggregate metrics for the file
    let aggregate = analyzer
        .analyze_file_aggregate(file_path)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;

    let functions = if include_functions {
        // Get function-level metrics
        let func_metrics = analyzer
            .analyze_file(file_path)
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;

        func_metrics
            .into_iter()
            .map(|f| FunctionComplexity {
                name: f.name,
                line: f.start_line,
                cyclomatic: f.metrics.cyclomatic,
                cognitive: f.metrics.cognitive,
                sloc: f.metrics.sloc,
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(ComplexityReport {
        file: file_path.to_str().unwrap_or_default().to_owned(),
        cyclomatic: aggregate.cyclomatic,
        cognitive: aggregate.cognitive,
        maintainability_index: aggregate.maintainability_index,
        sloc: aggregate.sloc,
        functions,
    })
}

// ---------------------------------------------------------------------------
// Linkme Registration
// ---------------------------------------------------------------------------
use mcb_domain::registry::services::{
    SERVICES_REGISTRY, ServiceBuilder, ServiceRegistryEntry, VALIDATION_SERVICE_NAME,
};

#[linkme::distributed_slice(SERVICES_REGISTRY)]
static VALIDATION_SERVICE_REGISTRY_ENTRY: ServiceRegistryEntry = ServiceRegistryEntry {
    name: VALIDATION_SERVICE_NAME,
    build: ServiceBuilder::Validation(|_context| {
        Ok(std::sync::Arc::new(InfraValidationService::new()))
    }),
};
