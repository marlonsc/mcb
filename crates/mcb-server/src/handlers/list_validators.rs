//! List Validators Tool Handler
//!
//! Handles the list_validators MCP tool call for listing available validators.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;

use mcb_domain::ports::services::ValidationServiceInterface;

use crate::args::ListValidatorsArgs;
use crate::formatter::ResponseFormatter;

/// Handler for listing available validators
#[derive(Clone)]
pub struct ListValidatorsHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl ListValidatorsHandler {
    /// Create a new list_validators handler
    pub fn new(validation_service: Arc<dyn ValidationServiceInterface>) -> Self {
        Self { validation_service }
    }

    /// Handle the list_validators tool request
    pub async fn handle(
        &self,
        Parameters(_args): Parameters<ListValidatorsArgs>,
    ) -> Result<CallToolResult, McpError> {
        match self.validation_service.list_validators().await {
            Ok(validators) => {
                let response = serde_json::json!({
                    "validators": validators,
                    "count": validators.len(),
                    "description": "Available validators for architecture validation"
                });
                ResponseFormatter::json_success(&response)
            }
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Error listing validators: {}",
                e
            ))])),
        }
    }
}
