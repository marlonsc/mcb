//! Processor implementation for hook events.
//!
//! Handles the execution logic for various lifecycle hooks.

use std::sync::Arc;

use mcb_domain::entities::memory::{MemoryFilter, ObservationType, OriginContext};
use mcb_domain::ports::services::MemoryServiceInterface;
use tracing::debug;

use super::types::{HookError, HookResult, PostToolUseContext, SessionStartContext};

/// Processor for tool execution hooks.
///
/// Handles lifecycle events like pre/post tool execution and session start.
pub struct HookProcessor {
    memory_service: Option<Arc<dyn MemoryServiceInterface>>,
}

impl HookProcessor {
    /// Create a new HookProcessor with optional memory service.
    pub fn new(memory_service: Option<Arc<dyn MemoryServiceInterface>>) -> Self {
        Self { memory_service }
    }

    /// Check if the processor is ready to handle events.
    pub fn is_ready(&self) -> bool {
        true
    }

    /// Process the PostToolUse hook event.
    ///
    /// Stores tool execution results as observations in memory.
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

        let project_id = context
            .metadata
            .get("project_id")
            .cloned()
            .unwrap_or_else(|| "default".to_string());

        let session_internal = context.session_id.clone().map(|id| id.into_string());

        let metadata = mcb_domain::entities::memory::ObservationMetadata {
            session_id: session_internal.clone(),
            origin_context: Some(OriginContext {
                project_id: Some(project_id.clone()),
                session_id: session_internal,
                tool_name: Some(context.tool_name.clone()),
                ..OriginContext::default()
            }),
            ..Default::default()
        };

        let mut tags = vec!["tool".to_string(), context.tool_name.clone()];
        if context.tool_output.is_error.unwrap_or(false) {
            tags.push("error".to_string());
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

    /// Process the SessionStart hook event.
    ///
    /// Injects relevant context from previous sessions.
    pub async fn process_session_start(&self, context: SessionStartContext) -> HookResult<()> {
        let memory_service = self
            .memory_service
            .as_ref()
            .ok_or(HookError::MemoryServiceUnavailable)?;

        debug!(
            session_id = %context.session_id,
            "Processing SessionStart hook"
        );

        let filter = MemoryFilter {
            id: None,
            project_id: None,
            tags: None,
            r#type: None,
            session_id: Some(context.session_id.as_str().to_string()),
            repo_id: None,
            time_range: None,
            branch: None,
            commit: None,
        };

        let _results = memory_service
            .memory_search("session context", Some(filter), 10)
            .await
            .map_err(|e| HookError::FailedToInjectContext(e.to_string()))?;

        debug!("SessionStart hook processed successfully");
        Ok(())
    }
}

impl Default for HookProcessor {
    fn default() -> Self {
        Self::new(None)
    }
}
