//! Handler for the `memory_store_execution` MCP tool

use crate::args::MemoryStoreExecutionArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    ExecutionMetadata, ExecutionType, ObservationMetadata, ObservationType,
};
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Handler for the MCP `memory_store_execution` tool.
pub struct StoreExecutionHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct StoreResult {
    observation_id: String,
    deduplicated: bool,
}

impl StoreExecutionHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryStoreExecutionArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let execution_type: ExecutionType = args
            .execution_type
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let execution_metadata = ExecutionMetadata {
            id: Uuid::new_v4().to_string(),
            command: args.command.clone(),
            exit_code: args.exit_code,
            duration_ms: args.duration_ms,
            success: args.success,
            execution_type: execution_type.clone(),
            coverage: args.coverage,
            files_affected: args.files_affected.clone(),
            output_summary: args.output_summary.clone(),
            warnings_count: args.warnings_count,
            errors_count: args.errors_count,
        };

        let content = build_execution_content(&execution_metadata);

        let vcs_context = VcsContext::capture();
        let branch = args.branch.or(vcs_context.branch);
        let commit = args.commit.or(vcs_context.commit);

        let tags = build_execution_tags(&execution_type, args.success);

        match self
            .memory_service
            .store_observation(
                content,
                ObservationType::Execution,
                tags,
                ObservationMetadata {
                    id: Uuid::new_v4().to_string(),
                    session_id: args.session_id,
                    repo_id: args.repo_id.or(vcs_context.repo_id),
                    file_path: None,
                    branch,
                    commit,
                    execution: Some(execution_metadata),
                    quality_gate: None,
                },
            )
            .await
        {
            Ok((id, deduplicated)) => {
                let result = StoreResult {
                    observation_id: id,
                    deduplicated,
                };
                ResponseFormatter::json_success(&result)
            }
            Err(_) => Ok(CallToolResult::error(vec![Content::text(
                "Failed to store execution",
            )])),
        }
    }
}

fn build_execution_tags(execution_type: &ExecutionType, success: bool) -> Vec<String> {
    let mut tags = vec!["execution".to_string(), execution_type.as_str().to_string()];
    if success {
        tags.push("success".to_string());
    } else {
        tags.push("failure".to_string());
    }
    tags
}

fn build_execution_content(execution: &ExecutionMetadata) -> String {
    let mut content = String::from("Execution Result\n");
    content.push_str("Type: ");
    content.push_str(execution.execution_type.as_str());
    content.push('\n');
    content.push_str("Command: ");
    content.push_str(&execution.command);
    content.push('\n');
    content.push_str("Success: ");
    content.push_str(if execution.success { "true" } else { "false" });
    content.push('\n');

    if let Some(exit_code) = execution.exit_code {
        content.push_str("Exit code: ");
        content.push_str(&exit_code.to_string());
        content.push('\n');
    }

    if let Some(duration_ms) = execution.duration_ms {
        content.push_str("Duration ms: ");
        content.push_str(&duration_ms.to_string());
        content.push('\n');
    }

    if let Some(coverage) = execution.coverage {
        content.push_str("Coverage: ");
        content.push_str(&coverage.to_string());
        content.push('\n');
    }

    if !execution.files_affected.is_empty() {
        content.push_str("Files affected: ");
        content.push_str(&execution.files_affected.join(", "));
        content.push('\n');
    }

    if let Some(warnings) = execution.warnings_count {
        content.push_str("Warnings: ");
        content.push_str(&warnings.to_string());
        content.push('\n');
    }

    if let Some(errors) = execution.errors_count {
        content.push_str("Errors: ");
        content.push_str(&errors.to_string());
        content.push('\n');
    }

    if let Some(summary) = &execution.output_summary {
        content.push_str("Output summary: ");
        content.push_str(summary);
        content.push('\n');
    }

    content
}
