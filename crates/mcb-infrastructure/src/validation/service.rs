//! Validation Service Implementation
//!
//! Implements `ValidationServiceInterface` using mcb-validate for
//! architecture validation.

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::services::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationReport, ValidationServiceInterface,
    ViolationEntry,
};
use std::path::Path;

/// Infrastructure validation service using mcb-validate
///
/// This is the real implementation that delegates to mcb-validate.
/// Named differently from mcb-application's stub to avoid REF002 violation.
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
        Ok(vec![
            "clean_architecture".into(),
            "solid".into(),
            "quality".into(),
            "organization".into(),
            "kiss".into(),
            "naming".into(),
            "documentation".into(),
            "performance".into(),
            "async_patterns".into(),
            "dependencies".into(),
            "patterns".into(),
            "tests".into(),
        ])
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
    use mcb_validate::{ArchitectureValidator, ValidationConfig};

    let config = ValidationConfig::new(workspace_root);
    let mut validator = ArchitectureValidator::with_config(config);

    let report = if let Some(names) = validators {
        let names_ref: Vec<&str> = names.iter().map(|s| s.as_str()).collect();
        validator
            .validate_named(&names_ref)
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?
    } else {
        validator
            .validate_all()
            .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))?
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
    // Return a list of known validation rules
    let all_rules = vec![
        // Clean Architecture rules
        RuleInfo {
            id: "CA001".into(),
            category: "clean_architecture".into(),
            severity: "error".into(),
            description: "Domain layer must not depend on infrastructure".into(),
            engine: "rust-validator".into(),
        },
        RuleInfo {
            id: "CA002".into(),
            category: "clean_architecture".into(),
            severity: "error".into(),
            description: "Application layer must not depend on infrastructure".into(),
            engine: "rust-validator".into(),
        },
        RuleInfo {
            id: "CA003".into(),
            category: "clean_architecture".into(),
            severity: "error".into(),
            description: "Ports must be defined in domain layer".into(),
            engine: "rust-validator".into(),
        },
        // SOLID rules
        RuleInfo {
            id: "SOLID001".into(),
            category: "solid".into(),
            severity: "warning".into(),
            description: "Single Responsibility Principle violation".into(),
            engine: "rust-validator".into(),
        },
        RuleInfo {
            id: "SOLID002".into(),
            category: "solid".into(),
            severity: "warning".into(),
            description: "Open/Closed Principle violation".into(),
            engine: "rust-validator".into(),
        },
        // Quality rules
        RuleInfo {
            id: "QUAL001".into(),
            category: "quality".into(),
            severity: "error".into(),
            description: "No unwrap() in production code".into(),
            engine: "rust-validator".into(),
        },
        RuleInfo {
            id: "QUAL002".into(),
            category: "quality".into(),
            severity: "error".into(),
            description: "No expect() in production code".into(),
            engine: "rust-validator".into(),
        },
        RuleInfo {
            id: "QUAL003".into(),
            category: "quality".into(),
            severity: "warning".into(),
            description: "Magic number detected".into(),
            engine: "rust-validator".into(),
        },
        // KISS rules
        RuleInfo {
            id: "KISS001".into(),
            category: "kiss".into(),
            severity: "warning".into(),
            description: "Function too long".into(),
            engine: "rca-metrics".into(),
        },
        RuleInfo {
            id: "KISS002".into(),
            category: "kiss".into(),
            severity: "warning".into(),
            description: "Cyclomatic complexity too high".into(),
            engine: "rca-metrics".into(),
        },
    ];

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

/// Null validation service for testing
pub struct NullValidationService;

impl NullValidationService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ValidationServiceInterface for NullValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        Ok(vec![
            "clean_architecture".into(),
            "solid".into(),
            "quality".into(),
        ])
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        Ok(ValidationReport {
            total_violations: 0,
            errors: 0,
            warnings: 0,
            infos: 0,
            violations: Vec::new(),
            passed: true,
        })
    }

    async fn get_rules(&self, _category: Option<&str>) -> Result<Vec<RuleInfo>> {
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        file_path: &Path,
        _include_functions: bool,
    ) -> Result<ComplexityReport> {
        Ok(ComplexityReport {
            file: file_path.to_string_lossy().to_string(),
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 100.0,
            sloc: 0,
            functions: Vec::new(),
        })
    }
}
