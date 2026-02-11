use mcb_server::args::{MemoryAction, MemoryArgs, MemoryResource};

/// Helper to create a base MemoryArgs with common defaults
pub(crate) fn create_base_memory_args(
    action: MemoryAction,
    resource: MemoryResource,
    data: Option<serde_json::Value>,
    ids: Option<Vec<String>>,
    session_id: Option<String>,
) -> MemoryArgs {
    MemoryArgs {
        action,
        resource,
        org_id: None,
        data,
        ids,
        project_id: Some("test-project".to_string()),
        repo_id: None,
        session_id: session_id.map(Into::into),
        tags: None,
        query: None,
        anchor_id: None,
        depth_before: None,
        depth_after: None,
        window_secs: None,
        observation_types: None,
        max_tokens: None,
        limit: None,
    }
}
