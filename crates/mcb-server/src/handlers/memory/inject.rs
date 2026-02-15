use std::sync::Arc;

use mcb_domain::ports::services::MemoryServiceInterface;
use mcb_infrastructure::project::context_resolver::capture_vcs_context;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use tracing::error;

use super::common::build_memory_filter;
use crate::args::MemoryArgs;
use crate::formatter::ResponseFormatter;
use crate::utils::mcp::tool_error;

/// Injects semantic memory context into the MCP tool result based on the provided filter.
#[tracing::instrument(skip_all)]
pub async fn inject_context(
    memory_service: &Arc<dyn MemoryServiceInterface>,
    args: &MemoryArgs,
) -> Result<CallToolResult, McpError> {
    let filter = build_memory_filter(args, None, None);
    let limit = args.limit.unwrap_or(10) as usize;
    let max_tokens = args.max_tokens.unwrap_or(2000);
    let vcs_context = capture_vcs_context();
    match memory_service
        .search_memories("", Some(filter), limit)
        .await
    {
        Ok(results) => {
            let mut context = String::new();
            let mut observation_ids = Vec::new();
            let max_chars = max_tokens * 4;
            for result in results {
                observation_ids.push(result.observation.id.clone());
                let entry = format!(
                    "[{}] {}: {}\n\n",
                    result.observation.r#type.as_str().to_uppercase(),
                    result.observation.id,
                    result.observation.content
                );
                if context.len() + entry.len() > max_chars {
                    break;
                }
                context.push_str(&entry);
            }
            ResponseFormatter::json_success(&serde_json::json!({
                "observation_count": observation_ids.len(),
                "observation_ids": observation_ids,
                "context": context,
                "estimated_tokens": context.len() / 4,
                "vcs_context": {
                    "branch": vcs_context.branch,
                    "commit": vcs_context.commit,
                }
            }))
        }
        Err(_e) => {
            error!("Failed to inject context");
            Ok(tool_error("Failed to inject context"))
        }
    }
}
