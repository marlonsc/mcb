//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Validate handler for code validation operations.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use mcb_domain::ports::ValidationServiceInterface;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use validator::Validate;

use crate::args::{ValidateAction, ValidateArgs, ValidateScope};
use crate::error_mapping::to_contextual_tool_error;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

/// Handler for code validation MCP tool operations.
#[derive(Clone)]
pub struct ValidateHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

handler_new!(ValidateHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
});

impl ValidateHandler {
    /// Handle a validate tool request.
    ///
    /// # Errors
    /// Returns an error when argument validation fails.
    #[tracing::instrument(skip_all)]
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ValidateArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("invalid arguments", None))?;

        match args.action {
            ValidateAction::Run => self.handle_run(&args).await,
            ValidateAction::ListRules => self.handle_list_rules(&args).await,
            ValidateAction::Analyze => self.handle_analyze(&args).await,
        }
    }

    async fn handle_run(&self, args: &ValidateArgs) -> Result<CallToolResult, McpError> {
        let path_str = Self::required_path(args, "Missing required parameter: path")?;
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

        self.run_validation_for_scope(args, &path, scope, timer)
            .await
    }

    async fn handle_list_rules(&self, args: &ValidateArgs) -> Result<CallToolResult, McpError> {
        let effective_category = args.category.as_deref().filter(|c| !c.trim().is_empty());
        if let Some(category) = effective_category {
            match self.validation_service.get_rules(Some(category)).await {
                Ok(rules) => ResponseFormatter::json_success(&serde_json::json!({
                    "rules": rules,
                    "count": rules.len(),
                    "filter": category,
                })),
                Err(e) => Ok(to_contextual_tool_error(e)),
            }
        } else {
            match self.validation_service.list_validators().await {
                Ok(validators) => ResponseFormatter::json_success(&serde_json::json!({
                    "validators": validators,
                    "count": validators.len(),
                    "description": "Available validation rules",
                })),
                Err(e) => Ok(to_contextual_tool_error(e)),
            }
        }
    }

    async fn handle_analyze(&self, args: &ValidateArgs) -> Result<CallToolResult, McpError> {
        let path_str = Self::required_path(args, "Missing required parameter: path for analyze")?;
        let path = PathBuf::from(path_str);
        if !path.exists() || !path.is_file() {
            return Ok(tool_error("Path must be an existing file"));
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
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }

    fn required_path<'a>(
        args: &'a ValidateArgs,
        missing_message: &'static str,
    ) -> Result<&'a str, McpError> {
        args.path
            .as_deref()
            .ok_or_else(|| McpError::invalid_params(missing_message, None))
    }

    async fn run_validation_for_scope(
        &self,
        args: &ValidateArgs,
        path: &std::path::Path,
        scope: ValidateScope,
        timer: Instant,
    ) -> Result<CallToolResult, McpError> {
        let result = match scope {
            ValidateScope::File => {
                self.validation_service
                    .validate_file(path, args.rules.as_deref())
                    .await
            }
            ValidateScope::Project => {
                self.validation_service
                    .validate(path, args.rules.as_deref(), None)
                    .await
            }
        };

        match result {
            Ok(report) => Ok(ResponseFormatter::format_validation_success(
                &report,
                path,
                timer.elapsed(),
            )),
            Err(e) => Ok(to_contextual_tool_error(e)),
        }
    }
}
