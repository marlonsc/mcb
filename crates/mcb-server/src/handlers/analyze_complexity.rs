//! Analyze Complexity Tool Handler
//!
//! Handles the analyze_complexity MCP tool call for getting code complexity metrics.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_domain::ports::services::ValidationServiceInterface;

use crate::args::AnalyzeComplexityArgs;
use crate::formatter::ResponseFormatter;

/// Handler for code complexity analysis
#[derive(Clone)]
pub struct AnalyzeComplexityHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl AnalyzeComplexityHandler {
    /// Create a new analyze_complexity handler
    pub fn new(validation_service: Arc<dyn ValidationServiceInterface>) -> Self {
        Self { validation_service }
    }

    /// Handle the analyze_complexity tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<AnalyzeComplexityArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let path = PathBuf::from(&args.path);
        if !path.exists() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "File does not exist: {}",
                path.display()
            ))]));
        }

        if !path.is_file() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Path is not a file: {}",
                path.display()
            ))]));
        }

        let timer = Instant::now();

        match self
            .validation_service
            .analyze_complexity(&path, args.include_functions)
            .await
        {
            Ok(report) => {
                let response = serde_json::json!({
                    "file": report.file,
                    "metrics": {
                        "cyclomatic": report.cyclomatic,
                        "cognitive": report.cognitive,
                        "maintainability_index": report.maintainability_index,
                        "sloc": report.sloc
                    },
                    "functions": report.functions,
                    "analysis_time_ms": timer.elapsed().as_millis()
                });
                ResponseFormatter::json_success(&response)
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error analyzing complexity for {}: {}",
                path.display(),
                e
            ))])),
        }
    }
}
