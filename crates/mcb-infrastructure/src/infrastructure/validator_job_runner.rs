//!
//! Validator job runner: centralizes validator execution via domain port.
//!
//! Runs validation jobs using [`ValidationServiceInterface`] and tracks them
//! with [`ValidationOperationsInterface`]. Can be extended to use Loco workers.

use std::path::Path;
use std::sync::Arc;

use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{
    ValidationOperationResult, ValidationOperationsInterface, ValidationServiceInterface,
    ValidatorJobRunner,
};
use mcb_domain::value_objects::OperationId;

/// Default validator job runner (in-process execution).
///
/// Submits validation jobs by spawning a background task that runs
/// [`ValidationServiceInterface::validate`] and updates
/// [`ValidationOperationsInterface`]. Ready to be backed by Loco queue when
/// workers are connected.
pub struct DefaultValidatorJobRunner {
    validation_ops: Arc<dyn ValidationOperationsInterface>,
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl DefaultValidatorJobRunner {
    /// Create a new runner with the given tracking and execution dependencies.
    #[must_use]
    pub fn new(
        validation_ops: Arc<dyn ValidationOperationsInterface>,
        validation_service: Arc<dyn ValidationServiceInterface>,
    ) -> Self {
        Self {
            validation_ops,
            validation_service,
        }
    }
}

impl ValidatorJobRunner for DefaultValidatorJobRunner {
    fn submit_validation_job(&self, workspace: &str, validators: &[String]) -> Result<OperationId> {
        let runtime = tokio::runtime::Handle::try_current().map_err(|e| {
            Error::invalid_argument(format!(
                "validation job submission requires an active tokio runtime: {e}"
            ))
        })?;

        let op_id = self.validation_ops.start_operation(workspace, validators);
        let ops = Arc::clone(&self.validation_ops);
        let svc = Arc::clone(&self.validation_service);
        let workspace_path = workspace.to_owned();
        let validators_vec = validators.to_vec();
        runtime.spawn(async move {
            let path = Path::new(&workspace_path);
            let names_opt: Option<&[String]> = if validators_vec.is_empty() {
                None
            } else {
                Some(validators_vec.as_slice())
            };
            match svc.validate(path, names_opt, None).await {
                Ok(report) => {
                    let result = ValidationOperationResult {
                        total_violations: report.total_violations,
                        errors: report.errors,
                        warnings: report.warnings,
                        passed: report.passed,
                    };
                    ops.complete_operation(&op_id, result);
                }
                Err(e) => {
                    mcb_domain::error!("validator_job_runner", "validation job failed", &e);
                    ops.complete_operation(
                        &op_id,
                        ValidationOperationResult {
                            total_violations: 1,
                            errors: 1,
                            warnings: 0,
                            passed: false,
                        },
                    );
                }
            }
        });
        Ok(op_id)
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Arc;

    use async_trait::async_trait;
    use mcb_domain::error::Result;
    use mcb_domain::ports::{
        ComplexityReport, FunctionComplexity, RuleInfo, ValidationOperationsInterface,
        ValidationReport, ValidationServiceInterface,
    };

    use crate::infrastructure::DefaultValidationOperations;

    use super::DefaultValidatorJobRunner;
    use super::ValidatorJobRunner;

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

    #[tokio::test]
    async fn submit_validation_job_completes_successfully() {
        let ops: Arc<dyn ValidationOperationsInterface> =
            Arc::new(DefaultValidationOperations::new());
        let svc: Arc<dyn ValidationServiceInterface> = Arc::new(SuccessValidationService);
        let runner = DefaultValidatorJobRunner::new(Arc::clone(&ops), svc);

        let op_id = runner
            .submit_validation_job(".", &["clean_architecture".to_owned()])
            .expect("job should be submitted");

        for _ in 0..40 {
            if let Some(op) = ops.get_operation(&op_id)
                && !ops.is_in_progress(&op_id)
            {
                assert_eq!(op.status, mcb_domain::ports::ValidationStatus::Completed);
                assert_eq!(op.result.map(|r| r.errors), Some(0));
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        panic!("operation did not complete in time");
    }

    #[tokio::test]
    async fn submit_validation_job_marks_failure_result() {
        let ops: Arc<dyn ValidationOperationsInterface> =
            Arc::new(DefaultValidationOperations::new());
        let svc: Arc<dyn ValidationServiceInterface> = Arc::new(FailingValidationService);
        let runner = DefaultValidatorJobRunner::new(Arc::clone(&ops), svc);

        let op_id = runner
            .submit_validation_job(".", &[])
            .expect("job should be submitted");

        for _ in 0..40 {
            if let Some(op) = ops.get_operation(&op_id)
                && !ops.is_in_progress(&op_id)
            {
                assert_eq!(op.status, mcb_domain::ports::ValidationStatus::Completed);
                let result = op.result.expect("result should be present");
                assert!(!result.passed);
                assert_eq!(result.errors, 1);
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(25)).await;
        }

        panic!("operation did not complete in time");
    }

    #[test]
    fn submit_validation_job_requires_runtime() {
        let ops: Arc<dyn ValidationOperationsInterface> =
            Arc::new(DefaultValidationOperations::new());
        let svc: Arc<dyn ValidationServiceInterface> = Arc::new(SuccessValidationService);
        let runner = DefaultValidatorJobRunner::new(ops, svc);

        let err = runner
            .submit_validation_job(".", &[])
            .expect_err("submission without runtime should fail");
        assert!(
            err.to_string().contains("active tokio runtime"),
            "unexpected error: {err}"
        );
    }
}
