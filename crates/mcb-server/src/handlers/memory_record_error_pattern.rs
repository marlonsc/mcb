//! Handler for the `memory_record_error_pattern` MCP tool

use crate::args::MemoryRecordErrorPatternArgs;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{ErrorPattern, ErrorPatternMatch};
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use std::sync::Arc;
use validator::Validate;

/// Handler for the MCP `memory_record_error_pattern` tool.
pub struct MemoryRecordErrorPatternHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

impl MemoryRecordErrorPatternHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryRecordErrorPatternArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let _error_pattern_types: Option<(ErrorPattern, ErrorPatternMatch)> = None;

        let _ = &self.memory_service;

        Ok(CallToolResult::error(vec![Content::text(
            "Error pattern memory is not implemented yet",
        )]))
    }
}
