//! Get Validation Rules Tool Handler
//!
//! Handles the get_validation_rules MCP tool call for retrieving available rules.

use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;
use validator::Validate;

use mcb_domain::ports::services::ValidationServiceInterface;

use crate::args::GetValidationRulesArgs;

/// Handler for getting validation rules
#[derive(Clone)]
pub struct GetValidationRulesHandler {
    validation_service: Arc<dyn ValidationServiceInterface>,
}

impl GetValidationRulesHandler {
    /// Create a new get_validation_rules handler
    pub fn new(validation_service: Arc<dyn ValidationServiceInterface>) -> Self {
        Self { validation_service }
    }

    /// Handle the get_validation_rules tool request
    pub async fn handle(
        &self,
        Parameters(args): Parameters<GetValidationRulesArgs>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(e) = args.validate() {
            return Err(McpError::invalid_params(
                format!("Invalid arguments: {}", e),
                None,
            ));
        }

        match self
            .validation_service
            .get_rules(args.category.as_deref())
            .await
        {
            Ok(rules) => {
                let response = serde_json::json!({
                    "rules": rules,
                    "count": rules.len(),
                    "filter": args.category.as_deref().unwrap_or("all")
                });

                let res = serde_json::to_string_pretty(&response);
                #[rustfmt::skip]
                let text = res.map_err(|_| McpError::internal_error("Failed to load validation rules", None))?;

                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Error getting validation rules",
            )])),
        }
    }
}
