//!
//! Validator job runner: centralizes validator execution via domain port.
//!
//! Runs validation jobs using [`ValidationServiceInterface`] and tracks them
//! with [`ValidationOperationsInterface`]. Can be extended to use Loco workers.

use std::path::Path;
use std::sync::Arc;

use mcb_domain::ports::{
    ValidationOperationResult, ValidationOperationsInterface, ValidationServiceInterface,
    ValidatorJobRunner,
};
use mcb_domain::value_objects::OperationId;
use tracing::instrument;

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
    #[instrument(skip(self), fields(workspace = %workspace))]
    fn submit_validation_job(
        &self,
        workspace: &str,
        validators: &[String],
    ) -> Result<OperationId, String> {
        let op_id = self.validation_ops.start_operation(workspace, validators);
        let ops = Arc::clone(&self.validation_ops);
        let svc = Arc::clone(&self.validation_service);
        let workspace_path = workspace.to_owned();
        let validators_vec = validators.to_vec();
        tokio::spawn(async move {
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
                    tracing::error!(error = %e, "validation job failed");
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
