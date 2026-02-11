//! Validation Service Implementation
//!
//! Implements `ValidationServiceInterface` using mcb-validate for
//! architecture validation.

use std::path::Path;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};

/// Infrastructure validation service using mcb-validate
pub struct InfraValidationService;

impl InfraValidationService {
    /// Create a new validation service
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
        use mcb_validate::ValidatorRegistry;

        Ok(ValidatorRegistry::standard_validator_names()
            .iter()
            .map(|name| (*name).to_string())
            .collect())
    }

    async fn validate_file(
        &self,
        file_path: &Path,
        validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        run_file_validation(file_path, validators)
    }

    async fn get_rules(&self, category: Option<&str>) -> Result<Vec<RuleInfo>> {
        get_validation_rules(category)
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
    use mcb_validate::{GenericReporter, ValidationConfig, ValidatorRegistry};

    let config = ValidationConfig::new(workspace_root);
    let registry = ValidatorRegistry::standard_for(workspace_root);

    let report = if let Some(names) = validators {
        let names_ref: Vec<&str> = names.iter().map(String::as_str).collect();
        let violations = registry
            .validate_named(&config, &names_ref)
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;
        GenericReporter::create_report(&violations, workspace_root.to_path_buf())
    } else {
        let violations = registry
            .validate_all(&config)
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;
        GenericReporter::create_report(&violations, workspace_root.to_path_buf())
    };

    Ok(convert_report(report, severity_filter))
}

fn convert_report(
    report: mcb_validate::GenericReport,
    severity_filter: Option<&str>,
) -> ValidationReport {
    let min_severity = match severity_filter {
        Some("error") => 0,
        Some("warning") => 1,
        _ => 2,
    };

    let all_violations: Vec<mcb_validate::ViolationEntry> = report
        .violations_by_category
        .into_values()
        .flatten()
        .collect();

    let violations: Vec<ViolationEntry> = all_violations
        .into_iter()
        .filter(|v| {
            let severity_level = match v.severity.as_str() {
                "ERROR" => 0,
                "WARNING" => 1,
                _ => 2,
            };
            severity_level <= min_severity
        })
        .map(|v| ViolationEntry {
            id: v.id,
            category: v.category,
            severity: v.severity,
            file: v.file.map(|p| p.to_string_lossy().to_string()),
            line: v.line,
            message: v.message,
            suggestion: v.suggestion,
        })
        .collect();

    let errors = violations.iter().filter(|v| v.severity == "ERROR").count();

    ValidationReport {
        total_violations: violations.len(),
        errors,
        warnings: violations
            .iter()
            .filter(|v| v.severity == "WARNING")
            .count(),
        infos: violations.iter().filter(|v| v.severity == "INFO").count(),
        violations,
        passed: errors == 0,
    }
}

fn run_file_validation(
    file_path: &Path,
    validators: Option<&[String]>,
) -> Result<ValidationReport> {
    // For single file validation, we need to find the workspace root
    // and run validation scoped to that file
    let workspace_root = find_workspace_root(file_path)
        .unwrap_or_else(|| file_path.parent().unwrap_or(file_path).to_path_buf());

    // Run standard validation - mcb-validate doesn't have single-file mode yet
    // So we run full validation and filter to the specific file
    let full_report = run_validation(&workspace_root, validators, None)?;

    let file_str = file_path.to_string_lossy().to_string();
    let file_violations: Vec<ViolationEntry> = full_report
        .violations
        .into_iter()
        .filter(|v| v.file.as_ref().is_some_and(|f| f.contains(&file_str)))
        .collect();

    let errors = file_violations
        .iter()
        .filter(|v| v.severity == "ERROR")
        .count();

    Ok(ValidationReport {
        total_violations: file_violations.len(),
        errors,
        warnings: file_violations
            .iter()
            .filter(|v| v.severity == "WARNING")
            .count(),
        infos: file_violations
            .iter()
            .filter(|v| v.severity == "INFO")
            .count(),
        violations: file_violations,
        passed: errors == 0,
    })
}

fn find_workspace_root(start: &Path) -> Option<std::path::PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };

    loop {
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists()
            && let Ok(content) = std::fs::read_to_string(&cargo_toml)
            && content.contains("[workspace]")
        {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn get_validation_rules(category: Option<&str>) -> Result<Vec<RuleInfo>> {
    let embedded = mcb_validate::EmbeddedRules::all_yaml();
    let mut loader = mcb_validate::YamlRuleLoader::from_embedded(&embedded)
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;
    let validated = loader
        .load_embedded_rules()
        .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?;

    let all_rules: Vec<RuleInfo> = validated
        .into_iter()
        .filter(|r| r.enabled)
        .map(|r| RuleInfo {
            id: r.id,
            category: r.category,
            severity: r.severity,
            description: r.description,
            engine: r.engine,
        })
        .collect();

    if let Some(cat) = category {
        Ok(all_rules
            .into_iter()
            .filter(|r| r.category == cat)
            .collect())
    } else {
        Ok(all_rules)
    }
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
        file: file_path.to_string_lossy().to_string(),
        cyclomatic: aggregate.cyclomatic,
        cognitive: aggregate.cognitive,
        maintainability_index: aggregate.maintainability_index,
        sloc: aggregate.sloc,
        functions,
    })
}
