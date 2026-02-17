//! Processor implementation for hook events.
//!
//! Handles the execution logic for various lifecycle hooks.

use std::sync::Arc;

use mcb_domain::entities::memory::{MemoryFilter, ObservationType, OriginContext};
use mcb_domain::ports::MemoryServiceInterface;
use mcb_domain::utils::mask_id;

use mcb_domain::utils::id as domain_id;
use tracing::debug;

use crate::constants::fields::TAG_TOOL;

use super::types::{
    HookError, HookResult, PostToolUseContext, SessionStartContext, ToolExecutionStatus,
};

/// Processor for tool execution hooks.
///
/// Handles lifecycle events like pre/post tool execution and session start.
pub struct HookProcessor {
    memory_service: Option<Arc<dyn MemoryServiceInterface>>,
}

impl HookProcessor {
    /// Create a new `HookProcessor` with optional memory service.
    #[must_use]
    pub fn new(memory_service: Option<Arc<dyn MemoryServiceInterface>>) -> Self {
        Self { memory_service }
    }

    /// Check if the processor is ready to handle events.
    #[must_use]
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Process the `PostToolUse` hook event.
    ///
    /// Stores tool execution results as observations in memory.
    ///
    /// # Errors
    /// Returns an error when memory service is unavailable or observation storage fails.
    pub async fn process_post_tool_use(&self, context: PostToolUseContext) -> HookResult<()> {
        let memory_service = self
            .memory_service
            .as_ref()
            .ok_or(HookError::MemoryServiceUnavailable)?;

        debug!(
            tool_name = %context.tool_name,
            status = ?context.status,
            "Processing PostToolUse hook"
        );

        let content = format!(
            "Tool '{}' executed with status: {:?}",
            context.tool_name, context.status
        );

        let project_id = match context.metadata.get("project_id").cloned() {
            Some(id) if !id.trim().is_empty() => id,
            _ => {
                debug!("PostToolUse hook skipped: missing project_id in metadata");
                return Ok(());
            }
        };

        let parent_session_hash = context
            .metadata
            .get("parent_session_id")
            .map(|parent| domain_id::correlate_id("parent_session", parent.as_str()));
        let delegated = context.metadata.get("delegated").and_then(|value| {
            match value.trim().to_ascii_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            }
        });

        let metadata = mcb_domain::entities::memory::ObservationMetadata {
            session_id: None,
            origin_context: Some(
                OriginContext::builder()
                    .project_id(Some(project_id.clone()))
                    .parent_session_id_correlation(parent_session_hash)
                    .tool_name(Some(context.tool_name.clone()))
                    .repo_id(context.metadata.get("repo_id").cloned())
                    .repo_path(context.metadata.get("repo_path").cloned())
                    .worktree_id(context.metadata.get("worktree_id").cloned())
                    .operator_id(context.metadata.get("operator_id").cloned())
                    .machine_id(context.metadata.get("machine_id").cloned())
                    .agent_program(context.metadata.get("agent_program").cloned())
                    .model_id(context.metadata.get("model_id").cloned())
                    .delegated(delegated)
                    .build(),
            ),
            ..Default::default()
        };

        let mut tags = vec![TAG_TOOL.to_owned(), context.tool_name.clone()];
        if context.status == ToolExecutionStatus::Error {
            tags.push("error".to_owned());
        }

        memory_service
            .store_observation(
                project_id,
                content,
                ObservationType::Execution,
                tags,
                metadata,
            )
            .await
            .map_err(|e| HookError::FailedToStoreObservation(e.to_string()))?;

        debug!("PostToolUse hook processed successfully");
        Ok(())
    }

    /// Process the `SessionStart` hook event.
    ///
    /// Injects relevant context from previous sessions.
    ///
    /// # Errors
    /// Returns an error when memory service is unavailable or context search fails.
    pub async fn process_session_start(&self, context: SessionStartContext) -> HookResult<()> {
        let memory_service = self
            .memory_service
            .as_ref()
            .ok_or(HookError::MemoryServiceUnavailable)?;
        let session_id_str = context.session_id.to_string();

        debug!(
            session_id = %mask_id(&session_id_str),
            "Processing SessionStart hook"
        );

        let filter = MemoryFilter {
            id: None,
            project_id: None,
            tags: None,
            r#type: None,
            session_id: Some(domain_id::correlate_id("session", &session_id_str)),
            parent_session_id: None,
            repo_id: None,
            time_range: None,
            branch: None,
            commit: None,
        };

        let results = memory_service
            .memory_search("session context", Some(filter), 10)
            .await
            .map_err(|e| HookError::FailedToInjectContext(e.to_string()))?;

        debug!(
            count = results.len(),
            "SessionStart hook processed successfully"
        );
        Ok(())
    }
}

impl Default for HookProcessor {
    fn default() -> Self {
        Self::new(None)
    }
}
