//! Row-to-entity conversion using the domain port [`SqlRow`].

use mcb_domain::constants::keys as schema;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, CheckpointType};
use mcb_domain::entities::memory::{
    Observation, ObservationMetadata, OriginContext, SessionSummary,
};
use mcb_domain::entities::project::Project;
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::SqlRow;
use mcb_domain::schema::COL_OBSERVATION_TYPE;

/// Helper function to extract a required string field from a row.
fn required_string(row: &dyn SqlRow, field_name: &str) -> Result<String> {
    row.try_get_string(field_name)?
        .ok_or_else(|| Error::memory(format!("Missing {}", field_name)))
}

/// Build an `Observation` from a port row.
pub fn row_to_observation(row: &dyn SqlRow) -> Result<Observation> {
    let tags_json: Option<String> = row.try_get_string("tags")?;
    let tags: Vec<String> = match tags_json {
        Some(json) => serde_json::from_str(&json)
            .map_err(|e| Error::memory_with_source("invalid observation tags JSON", e))?,
        None => Vec::new(),
    };

    let obs_type_str: String = row
        .try_get_string(COL_OBSERVATION_TYPE)?
        .unwrap_or_else(|| "context".to_string());
    let observation_type = obs_type_str
        .parse()
        .map_err(|e| Error::memory(format!("Invalid observation_type: {e}")))?;

    let metadata_json: Option<String> = row.try_get_string("metadata")?;
    let metadata: ObservationMetadata = match metadata_json {
        Some(json) => serde_json::from_str(&json)
            .map_err(|e| Error::memory_with_source("invalid observation metadata JSON", e))?,
        None => return Err(Error::memory("Missing metadata")),
    };

    Ok(Observation {
        id: required_string(row, schema::ID)?,
        project_id: required_string(row, "project_id")?,
        content: required_string(row, "content")?,
        content_hash: required_string(row, "content_hash")?,
        tags,
        r#type: observation_type,
        metadata,
        created_at: row
            .try_get_i64(schema::CREATED_AT)?
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
    let origin_context_json: Option<String> = row.try_get_string("origin_context")?;

    Ok(SessionSummary {
        id: required_string(row, "id")?,
        project_id: required_string(row, "project_id")?,
        session_id: required_string(row, "session_id")?,
        topics: match topics_json {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| Error::memory_with_source("invalid session summary topics JSON", e))?,
            None => Vec::new(),
        },
        decisions: match decisions_json {
            Some(json) => serde_json::from_str(&json).map_err(|e| {
                Error::memory_with_source("invalid session summary decisions JSON", e)
            })?,
            None => Vec::new(),
        },
        next_steps: match next_steps_json {
            Some(json) => serde_json::from_str(&json).map_err(|e| {
                Error::memory_with_source("invalid session summary next_steps JSON", e)
            })?,
            None => Vec::new(),
        },
        key_files: match key_files_json {
            Some(json) => serde_json::from_str(&json).map_err(|e| {
                Error::memory_with_source("invalid session summary key_files JSON", e)
            })?,
            None => Vec::new(),
        },
        origin_context: match origin_context_json {
            Some(json) => serde_json::from_str::<Option<OriginContext>>(&json).map_err(|e| {
                Error::memory_with_source("invalid session summary origin_context JSON", e)
            })?,
            None => None,
        },
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
            .try_get_string(schema::ID)?
            .ok_or_else(|| Error::memory("Missing id"))?,
        session_summary_id: row
            .try_get_string(schema::SESSION_SUMMARY_ID)?
            .ok_or_else(|| Error::memory("Missing session_summary_id"))?,
        agent_type,
        model: row
            .try_get_string(schema::MODEL)?
            .ok_or_else(|| Error::memory("Missing model"))?,
        parent_session_id: row.try_get_string(schema::PARENT_SESSION_ID)?,
        started_at: row
            .try_get_i64(schema::STARTED_AT)?
            .ok_or_else(|| Error::memory("Missing started_at"))?,
        ended_at: row.try_get_i64(schema::ENDED_AT)?,
        duration_ms: row.try_get_i64(schema::DURATION_MS)?,
        status,
        prompt_summary: row.try_get_string(schema::PROMPT_SUMMARY)?,
        result_summary: row.try_get_string(schema::RESULT_SUMMARY)?,
        token_count: row.try_get_i64(schema::TOKEN_COUNT)?,
        tool_calls_count: row.try_get_i64(schema::TOOL_CALLS_COUNT)?,
        delegations_count: row.try_get_i64(schema::DELEGATIONS_COUNT)?,
        project_id: row.try_get_string("project_id")?,
        worktree_id: row.try_get_string("worktree_id")?,
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

/// Build a `Project` from a port row.
pub fn row_to_project(row: &dyn SqlRow) -> Result<Project> {
    Ok(Project {
        id: required_string(row, "id")?,
        org_id: required_string(row, "org_id")?,
        name: required_string(row, "name")?,
        path: required_string(row, "path")?,
        created_at: row
            .try_get_i64("created_at")?
            .ok_or_else(|| Error::memory("Missing created_at"))?,
        updated_at: row
            .try_get_i64("updated_at")?
            .ok_or_else(|| Error::memory("Missing updated_at"))?,
    })
}
