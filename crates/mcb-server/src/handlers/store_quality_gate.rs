//! Handler for the `memory_store_quality_gate` MCP tool

use crate::args::MemoryStoreQualityGateArgs;
use crate::formatter::ResponseFormatter;
use mcb_application::ports::MemoryServiceInterface;
use mcb_domain::entities::memory::{
    ObservationMetadata, ObservationType, QualityGateResult, QualityGateStatus,
};
use mcb_domain::utils::vcs_context::VcsContext;
use rmcp::ErrorData as McpError;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;
use validator::Validate;

/// Handler for the MCP `memory_store_quality_gate` tool.
pub struct StoreQualityGateHandler {
    memory_service: Arc<dyn MemoryServiceInterface>,
}

#[derive(Serialize)]
struct StoreResult {
    observation_id: String,
    deduplicated: bool,
}

impl StoreQualityGateHandler {
    pub fn new(memory_service: Arc<dyn MemoryServiceInterface>) -> Self {
        Self { memory_service }
    }

    pub async fn handle(
        &self,
        Parameters(args): Parameters<MemoryStoreQualityGateArgs>,
    ) -> Result<CallToolResult, McpError> {
        args.validate()
            .map_err(|_| McpError::invalid_params("Invalid parameters", None))?;

        let status: QualityGateStatus = args
            .status
            .parse()
            .map_err(|e: String| McpError::invalid_params(e, None))?;

        let quality_gate = QualityGateResult {
            id: Uuid::new_v4().to_string(),
            gate_name: args.gate_name.clone(),
            status: status.clone(),
            message: args.message.clone(),
            timestamp: args.timestamp,
            execution_id: args.execution_id.clone(),
        };

        let content = build_quality_gate_content(&quality_gate);

        let vcs_context = VcsContext::capture();
        let branch = args.branch.or(vcs_context.branch);
        let commit = args.commit.or(vcs_context.commit);

        let tags = build_quality_gate_tags(&status);

        match self
            .memory_service
            .store_observation(
                content,
                ObservationType::QualityGate,
                tags,
                ObservationMetadata {
                    id: Uuid::new_v4().to_string(),
                    session_id: args.session_id,
                    repo_id: args.repo_id.or(vcs_context.repo_id),
                    file_path: None,
                    branch,
                    commit,
                    execution: None,
                    quality_gate: Some(quality_gate),
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
                "Failed to store quality gate result",
            )])),
        }
    }
}

fn build_quality_gate_tags(status: &QualityGateStatus) -> Vec<String> {
    vec!["quality_gate".to_string(), status.as_str().to_string()]
}

fn build_quality_gate_content(quality_gate: &QualityGateResult) -> String {
    let mut content = String::from("Quality Gate Result\n");
    content.push_str("Gate: ");
    content.push_str(&quality_gate.gate_name);
    content.push('\n');
    content.push_str("Status: ");
    content.push_str(quality_gate.status.as_str());
    content.push('\n');
    content.push_str("Timestamp: ");
    content.push_str(&quality_gate.timestamp.to_rfc3339());
    content.push('\n');

    if let Some(message) = &quality_gate.message {
        content.push_str("Message: ");
        content.push_str(message);
        content.push('\n');
    }

    if let Some(execution_id) = &quality_gate.execution_id {
        content.push_str("Execution ID: ");
        content.push_str(execution_id);
        content.push('\n');
    }

    content
}
