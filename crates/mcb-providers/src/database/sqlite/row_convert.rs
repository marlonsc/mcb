//! Row-to-entity conversion using the domain port [`SqlRow`].

use mcb_domain::entities::agent::{
    AgentSession, AgentSessionStatus, AgentType, Checkpoint, CheckpointType, Delegation, ToolCall,
};
use mcb_domain::entities::memory::{
    Observation, ObservationMetadata, ObservationType, SessionSummary,
};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::SqlRow;

/// Build an `Observation` from a port row.
pub fn row_to_observation(row: &dyn SqlRow) -> Result<Observation> {
    let tags_json: Option<String> = row.try_get_string("tags")?;
    let tags: Vec<String> = tags_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let obs_type_str: String = row
        .try_get_string("observation_type")?
        .unwrap_or_else(|| "context".to_string());
    let observation_type = obs_type_str.parse().unwrap_or(ObservationType::Context);

    let metadata_json: Option<String> = row.try_get_string("metadata")?;
    let metadata: ObservationMetadata = metadata_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Ok(Observation {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        project_id: row
            .try_get_string("project_id")?
            .ok_or_else(|| Error::memory("Missing project_id"))?,
        content: row
            .try_get_string("content")?
            .ok_or_else(|| Error::memory("Missing content"))?,
        content_hash: row
            .try_get_string("content_hash")?
            .ok_or_else(|| Error::memory("Missing content_hash"))?,
        tags,
        observation_type,
        metadata,
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
        embedding_id: row.try_get_string("embedding_id")?,
    })
}

/// Build a `SessionSummary` from a port row.
pub fn row_to_session_summary(row: &dyn SqlRow) -> Result<SessionSummary> {
    let topics_json: Option<String> = row.try_get_string("topics")?;
    let decisions_json: Option<String> = row.try_get_string("decisions")?;
    let next_steps_json: Option<String> = row.try_get_string("next_steps")?;
    let key_files_json: Option<String> = row.try_get_string("key_files")?;

    Ok(SessionSummary {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        project_id: row
            .try_get_string("project_id")?
            .ok_or_else(|| Error::memory("Missing project_id"))?,
        session_id: row
            .try_get_string("session_id")?
            .ok_or_else(|| Error::memory("Missing session_id"))?,
        topics: topics_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        decisions: decisions_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        next_steps: next_steps_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        key_files: key_files_json
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default(),
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
    })
}

/// Build an `AgentSession` from a port row.
pub fn row_to_agent_session(row: &dyn SqlRow) -> Result<AgentSession> {
    let agent_type_str = row
        .try_get_string("agent_type")?
        .ok_or_else(|| Error::memory("Missing agent_type"))?;
    let agent_type = agent_type_str
        .parse()
        .map_err(|e| Error::memory(format!("Invalid agent_type: {e}")))?;

    let status_str = row
        .try_get_string("status")?
        .ok_or_else(|| Error::memory("Missing status"))?;
    let status = status_str
        .parse()
        .map_err(|e| Error::memory(format!("Invalid status: {e}")))?;

    Ok(AgentSession {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        session_summary_id: row
            .try_get_string("session_summary_id")?
            .ok_or_else(|| Error::memory("Missing session_summary_id"))?,
        agent_type,
        model: row
            .try_get_string("model")?
            .ok_or_else(|| Error::memory("Missing model"))?,
        parent_session_id: row.try_get_string("parent_session_id")?,
        started_at: row
            .try_get_i64("started_at")?
            .ok_or_else(|| Error::memory("Missing started_at"))?,
        ended_at: row.try_get_i64("ended_at")?,
        duration_ms: row.try_get_i64("duration_ms")?,
        status,
        prompt_summary: row.try_get_string("prompt_summary")?,
        result_summary: row.try_get_string("result_summary")?,
        token_count: row.try_get_i64("token_count")?,
        tool_calls_count: row.try_get_i64("tool_calls_count")?,
        delegations_count: row.try_get_i64("delegations_count")?,
    })
}

/// Build a `Delegation` from a port row.
pub fn row_to_delegation(row: &dyn SqlRow) -> Result<Delegation> {
    let success = row
        .try_get_i64("success")?
        .ok_or_else(|| Error::memory("Missing success"))?
        != 0;

    Ok(Delegation {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        parent_session_id: row
            .try_get_string("parent_session_id")?
            .ok_or_else(|| Error::memory("Missing parent_session_id"))?,
        child_session_id: row
            .try_get_string("child_session_id")?
            .ok_or_else(|| Error::memory("Missing child_session_id"))?,
        prompt: row
            .try_get_string("prompt")?
            .ok_or_else(|| Error::memory("Missing prompt"))?,
        prompt_embedding_id: row.try_get_string("prompt_embedding_id")?,
        result: row.try_get_string("result")?,
        success,
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
        completed_at: row.try_get_i64("completed_at")?,
        duration_ms: row.try_get_i64("duration_ms")?,
    })
}

/// Build a `ToolCall` from a port row.
pub fn row_to_tool_call(row: &dyn SqlRow) -> Result<ToolCall> {
    let success = row
        .try_get_i64("success")?
        .ok_or_else(|| Error::memory("Missing success"))?
        != 0;

    Ok(ToolCall {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        session_id: row
            .try_get_string("session_id")?
            .ok_or_else(|| Error::memory("Missing session_id"))?,
        tool_name: row
            .try_get_string("tool_name")?
            .ok_or_else(|| Error::memory("Missing tool_name"))?,
        params_summary: row.try_get_string("params_summary")?,
        success,
        error_message: row.try_get_string("error_message")?,
        duration_ms: row.try_get_i64("duration_ms")?,
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
    })
}

/// Build a `Checkpoint` from a port row.
pub fn row_to_checkpoint(row: &dyn SqlRow) -> Result<Checkpoint> {
    let checkpoint_type_str = row
        .try_get_string("checkpoint_type")?
        .ok_or_else(|| Error::memory("Missing checkpoint_type"))?;
    let checkpoint_type: CheckpointType = checkpoint_type_str
        .parse()
        .map_err(|e| Error::memory(format!("Invalid checkpoint_type: {e}")))?;

    let snapshot_json = row
        .try_get_string("snapshot_data")?
        .ok_or_else(|| Error::memory("Missing snapshot_data"))?;
    let snapshot_data = serde_json::from_str(&snapshot_json)
        .map_err(|e| Error::memory_with_source("deserialize checkpoint snapshot", e))?;

    let expired = row
        .try_get_i64("expired")?
        .ok_or_else(|| Error::memory("Missing expired"))?
        != 0;

    Ok(Checkpoint {
        id: row
            .try_get_string("id")?
            .ok_or_else(|| Error::memory("Missing id"))?,
        session_id: row
            .try_get_string("session_id")?
            .ok_or_else(|| Error::memory("Missing session_id"))?,
        checkpoint_type,
        description: row
            .try_get_string("description")?
            .ok_or_else(|| Error::memory("Missing description"))?,
        snapshot_data,
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
        restored_at: row.try_get_i64("restored_at")?,
        expired,
    })
}
