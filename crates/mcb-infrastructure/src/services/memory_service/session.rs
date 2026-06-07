//! Session summary creation and management.
//!
//! Handles storing high-level summaries of agent sessions.

use mcb_domain::entities::memory::OriginContext;
use mcb_domain::entities::memory::SessionSummary;
use mcb_domain::error::Result;
use mcb_domain::ports::CreateSessionSummaryInput;
use mcb_domain::utils::id;
use mcb_domain::utils::time as domain_time;

use super::MemoryServiceImpl;

impl MemoryServiceImpl {
    /// Create and store a session summary.
    pub(crate) async fn create_session_summary_impl(
        &self,
        input: CreateSessionSummaryInput,
    ) -> Result<String> {
        let session_id = input.session_id.to_string();
        let timestamp = domain_time::epoch_secs_i64()?;
        let project_id = if input.project_id.trim().is_empty() {
            self.project_id.clone()
        } else {
            input.project_id
        };
        let summary = SessionSummary {
            id: id::generate().to_string(),
            project_id: project_id.clone(),
            org_id: input.org_id,
            session_id: session_id.clone(),
            topics: input.topics,
            decisions: input.decisions,
            next_steps: input.next_steps,
            key_files: input.key_files,
            origin_context: Some(
                input.origin_context.unwrap_or(
                    OriginContext::builder()
                        .project_id(Some(project_id))
                        .session_id(Some(session_id))
                        .timestamp(Some(timestamp))
                        .build(),
                ),
            ),
            created_at: timestamp,
        };

        self.repository.store_session_summary(&summary).await?;
        Ok(summary.id)
    }
}
