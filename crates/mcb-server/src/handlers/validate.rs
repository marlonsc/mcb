//! Validate handler for code validation operations.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mcb_domain::ports::services::ValidationServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use validator::Validate;

use crate::args::{ValidateAction, ValidateArgs, ValidateScope};
use crate::formatter::ResponseFormatter;

/// Handler for code validation MCP tool operations.
#[derive(Clone)]
pub struct ValidateHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl ValidateHandler {
    pub fn new(validation_service: Arc<dyn ValidationServiceInterface>) -> Self {
        Self { validation_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<ValidateArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|e| McpError::invalid_params(format!("Invalid arguments: {e}"), None))?;

        match args.action {
            ValidateAction::Run => {
                let path_str = args.path.as_ref().ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: path", None)
                })?;
                let path = PathBuf::from(path_str);
                if !path.exists() {
                    return Ok(ResponseFormatter::format_validation_error(
                        &format!("Path does not exist: {path_str}"),
                        &path,
                    ));
                }
                let timer = Instant::now();
                let scope = args.scope.unwrap_or_else(|| {
                    if path.is_file() {
                        ValidateScope::File
                    } else {
                        ValidateScope::Project
                    }
                });
                match scope {
                    ValidateScope::File => match self
                        .validation_service
                        .validate_file(&path, args.rules.as_deref())
                        .await
                    {
                        Ok(report) => Ok(ResponseFormatter::format_validation_success(
                            &report,
                            &path,
                            timer.elapsed(),
                        )),
                        Err(e) => Ok(ResponseFormatter::format_validation_error(
                            &format!("Validation failed for file {}: {}", path.display(), e),
                            &path,
                        )),
                    },
                    ValidateScope::Project => match self
                        .validation_service
                        .validate(&path, args.rules.as_deref(), None)
                        .await
                    {
                        Ok(report) => Ok(ResponseFormatter::format_validation_success(
                            &report,
                            &path,
                            timer.elapsed(),
                        )),
                        Err(e) => Ok(ResponseFormatter::format_validation_error(
                            &format!("Project validation failed for {}: {}", path.display(), e),
                            &path,
                        )),
                    },
                }
            }
            ValidateAction::ListRules => {
                if let Some(ref category) = args.category {
                    match self
                        .validation_service
                        .get_rules(Some(category.as_str()))
                        .await
                    {
                        Ok(rules) => ResponseFormatter::json_success(&serde_json::json!({
                            "rules": rules,
                            "count": rules.len(),
                            "filter": category,
                        })),
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to get validation rules: {}",
                            e
                        ))])),
                    }
                } else {
                    match self.validation_service.list_validators().await {
                        Ok(validators) => ResponseFormatter::json_success(&serde_json::json!({
                            "validators": validators,
                            "count": validators.len(),
                            "description": "Available validation rules",
                        })),
                        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                            "Failed to list validators: {}",
                            e
                        ))])),
                    }
                }
            }
            ValidateAction::Analyze => {
                let path_str = args.path.as_ref().ok_or_else(|| {
                    McpError::invalid_params("Missing required parameter: path for analyze", None)
                })?;
                let path = PathBuf::from(path_str);
                if !path.exists() || !path.is_file() {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "Path must be an existing file",
                    )]));
                }
                let timer = Instant::now();
                match self
                    .validation_service
                    .analyze_complexity(&path, true)
                    .await
                {
                    Ok(report) => ResponseFormatter::json_success(&serde_json::json!({
                        "path": path_str,
                        "cyclomatic": report.cyclomatic,
                        "cognitive": report.cognitive,
                        "maintainability_index": report.maintainability_index,
                        "sloc": report.sloc,
                        "functions": report.functions,
                        "analysis_time_ms": timer.elapsed().as_millis(),
                    })),
                    Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                        "Failed to analyze complexity: {}",
                        e
                    ))])),
                }
            }
        }
    }
}
