//! Unit tests for `DefaultValidatorJobRunner`.
//!
//! Extracted from `src/infrastructure/validator_job_runner.rs` inline tests
//! to keep source files focused on production code.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::error::Result;
use mcb_domain::ports::ValidatorJobRunner;
use mcb_domain::ports::{
    ComplexityReport, FunctionComplexity, RuleInfo, ValidationOperationsInterface,
    ValidationReport, ValidationServiceInterface,
};
use mcb_domain::registry::admin_operations::{
    ValidationOperationsProviderConfig, resolve_validation_operations_provider,
};
use mcb_domain::utils::tests::utils::TestResult;
use mcb_infrastructure::infrastructure::DefaultValidatorJobRunner;
use rstest::{fixture, rstest};

struct SuccessValidationService;

#[async_trait]
impl ValidationServiceInterface for SuccessValidationService {
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
        Ok(vec!["clean_architecture".to_owned()])
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
        include_functions: bool,
    ) -> Result<ComplexityReport> {
        let functions = if include_functions {
            vec![FunctionComplexity {
                name: "test_fn".to_owned(),
                line: 1,
                cyclomatic: 1.0,
                cognitive: 1.0,
                sloc: 1,
            }]
        } else {
            Vec::new()
        };

        Ok(ComplexityReport {
            file: file_path.display().to_string(),
            cyclomatic: 1.0,
            cognitive: 1.0,
            maintainability_index: 100.0,
            sloc: 1,
            functions,
        })
    }
}

struct FailingValidationService;

#[async_trait]
impl ValidationServiceInterface for FailingValidationService {
    async fn validate(
        &self,
        _workspace_root: &Path,
        _validators: Option<&[String]>,
        _severity_filter: Option<&str>,
    ) -> Result<ValidationReport> {
        Err(mcb_domain::error::Error::internal(
            "simulated validation failure",
        ))
    }

    async fn list_validators(&self) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn validate_file(
        &self,
        _file_path: &Path,
        _validators: Option<&[String]>,
    ) -> Result<ValidationReport> {
        Err(mcb_domain::error::Error::internal(
            "simulated validation failure",
        ))
    }

    async fn get_rules(&self, _category: Option<&str>) -> Result<Vec<RuleInfo>> {
        Ok(Vec::new())
    }

    async fn analyze_complexity(
        &self,
        _file_path: &Path,
        _include_functions: bool,
    ) -> Result<ComplexityReport> {
        Ok(ComplexityReport {
            file: "test".to_owned(),
            cyclomatic: 0.0,
            cognitive: 0.0,
            maintainability_index: 0.0,
            sloc: 0,
            functions: Vec::new(),
        })
    }
}

#[fixture]
fn validation_ops() -> TestResult<Arc<dyn ValidationOperationsInterface>> {
    resolve_validation_operations_provider(&ValidationOperationsProviderConfig::new("default"))
        .map_err(Into::into)
}

#[rstest]
#[tokio::test]
async fn submit_validation_job_completes_successfully(
    validation_ops: TestResult<Arc<dyn ValidationOperationsInterface>>,
) -> TestResult {
    let ops = validation_ops?;
    let svc: Arc<dyn ValidationServiceInterface> = Arc::new(SuccessValidationService);
    let runner = DefaultValidatorJobRunner::new(Arc::clone(&ops), svc);

    let op_id = runner.submit_validation_job(".", &["clean_architecture".to_owned()])?;

    for _ in 0..40 {
        if let Some(op) = ops.get_operation(&op_id)
            && !ops.is_in_progress(&op_id)
        {
            assert_eq!(op.status, mcb_domain::ports::ValidationStatus::Completed);
            assert_eq!(op.result.map(|r| r.errors), Some(0));
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    Err("operation did not complete in time".into())
}

#[rstest]
#[tokio::test]
async fn submit_validation_job_marks_failure_result(
    validation_ops: TestResult<Arc<dyn ValidationOperationsInterface>>,
) -> TestResult {
    let ops = validation_ops?;
    let svc: Arc<dyn ValidationServiceInterface> = Arc::new(FailingValidationService);
    let runner = DefaultValidatorJobRunner::new(Arc::clone(&ops), svc);

    let op_id = runner.submit_validation_job(".", &[])?;

    for _ in 0..40 {
        if let Some(op) = ops.get_operation(&op_id)
            && !ops.is_in_progress(&op_id)
        {
            assert_eq!(op.status, mcb_domain::ports::ValidationStatus::Completed);
            let result = op.result.ok_or("result should be present")?;
            assert!(!result.passed);
            assert_eq!(result.errors, 1);
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }

    Err("operation did not complete in time".into())
}

#[rstest]
#[test]
fn submit_validation_job_requires_runtime(
    validation_ops: TestResult<Arc<dyn ValidationOperationsInterface>>,
) -> TestResult {
    let ops = validation_ops?;
    let svc: Arc<dyn ValidationServiceInterface> = Arc::new(SuccessValidationService);
    let runner = DefaultValidatorJobRunner::new(ops, svc);

    let err = runner
        .submit_validation_job(".", &[])
        .expect_err("submission without runtime should fail");
    assert!(
        err.to_string().contains("active tokio runtime"),
        "unexpected error: {err}"
    );
    Ok(())
}
