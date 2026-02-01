//! Validate File Tool Handler
//!
//! Handles the validate_file MCP tool call for running architecture
//! validation rules on a single file.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use validator::Validate;

use mcb_domain::ports::services::ValidationServiceInterface;

use crate::args::ValidateFileArgs;
use crate::formatter::ResponseFormatter;

/// Handler for single-file validation operations
#[derive(Clone)]
pub struct ValidateFileHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl ValidateFileHandler {
    /// Create a new validate_file handler
    pub fn new(validation_service: Arc<dyn ValidationServiceInterface>) -> Self {
        Self { validation_service }
    }

    /// Handle the validate_file tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<ValidateFileArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        let path = PathBuf::from(&args.path);
        if !path.exists() {
            return Ok(ResponseFormatter::format_validation_error(
                "Specified path does not exist",
                &path,
            ));
        }
        if !path.is_file() {
            return Ok(ResponseFormatter::format_validation_error(
                "Specified path is not a file",
                &path,
            ));
        }

        let timer = Instant::now();
        let validators_ref: Option<Vec<String>> = args.validators;

        match self
            .validation_service
            .validate_file(&path, validators_ref.as_deref())
            .await
        {
            Ok(report) => Ok(ResponseFormatter::format_validation_success(
                &report,
                &path,
                timer.elapsed(),
            )),
            Err(e) => Ok(ResponseFormatter::format_validation_error(
                &e.to_string(),
                &path,
            )),
        }
    }
}
