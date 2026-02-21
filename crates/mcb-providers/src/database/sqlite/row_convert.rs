//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
//! Row-to-entity conversion using the domain port [`SqlRow`].

use mcb_domain::constants::keys as schema;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, CheckpointType};
use mcb_domain::entities::issue::{IssueComment, IssueLabel};
use mcb_domain::entities::memory::{Observation, ObservationMetadata, SessionSummary};
use mcb_domain::entities::project::Project;
use mcb_domain::entities::project::{IssueStatus, IssueType, ProjectIssue};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::SqlRow;
use mcb_domain::schema::COL_OBSERVATION_TYPE;
use std::str::FromStr;

/// Helper function to extract a required string field from a row.
fn required_string(row: &dyn SqlRow, field_name: &str) -> Result<String> {
    row.try_get_string(field_name)?
        .ok_or_else(|| Error::memory(format!("Missing {field_name}")))
}

/// Helper function to extract a required i64 field from a row.
fn required_i64(row: &dyn SqlRow, field_name: &str) -> Result<i64> {
    row.try_get_i64(field_name)?
        .ok_or_else(|| Error::memory(format!("Missing {field_name}")))
}

/// Parse an optional JSON column into a `Vec<T>`, defaulting to empty.
fn json_vec<T: serde::de::DeserializeOwned>(
    row: &dyn SqlRow,
    field: &str,
    err_ctx: &str,
) -> Result<Vec<T>> {
    match row.try_get_string(field)? {
        Some(json) => {
            serde_json::from_str(&json).map_err(|e| Error::memory_with_source(err_ctx, e))
        }
        None => Ok(Vec::new()),
    }
}

/// Parse an optional JSON column into `Option<T>`, defaulting to `None`.
fn json_opt<T: serde::de::DeserializeOwned>(
    row: &dyn SqlRow,
    field: &str,
    err_ctx: &str,
) -> Result<Option<T>> {
    match row.try_get_string(field)? {
        Some(json) => {
            serde_json::from_str(&json).map_err(|e| Error::memory_with_source(err_ctx, e))
        }
        None => Ok(None),
    }
}

/// Build an `Observation` from a port row.
pub fn row_to_observation(row: &dyn SqlRow) -> Result<Observation> {
    let tags: Vec<String> = json_vec(row, "tags", "invalid observation tags JSON")?;

    let obs_type_str: String = row
        .try_get_string(COL_OBSERVATION_TYPE)?
        .unwrap_or_else(|| "context".to_owned());
    let observation_type = obs_type_str
        .parse()
        .map_err(|e| Error::memory(format!("Invalid observation_type: {e}")))?;

    let metadata: ObservationMetadata =
        json_opt(row, "metadata", "invalid observation metadata JSON")?
            .ok_or_else(|| Error::memory("Missing metadata"))?;

    Ok(Observation {
        id: required_string(row, schema::ID)?,
        project_id: required_string(row, "project_id")?,
        content: required_string(row, "content")?,
        content_hash: required_string(row, "content_hash")?,
        tags,
        r#type: observation_type,
        metadata,
        created_at: required_i64(row, schema::CREATED_AT)?,
        embedding_id: row.try_get_string("embedding_id")?,
    })
}

/// Build a `SessionSummary` from a port row.
pub fn row_to_session_summary(row: &dyn SqlRow) -> Result<SessionSummary> {
    Ok(SessionSummary {
        id: required_string(row, "id")?,
        project_id: required_string(row, "project_id")?,
        org_id: required_string(row, "org_id")?,
        session_id: required_string(row, "session_id")?,
        topics: json_vec(row, "topics", "invalid session summary topics JSON")?,
        decisions: json_vec(row, "decisions", "invalid session summary decisions JSON")?,
        next_steps: json_vec(row, "next_steps", "invalid session summary next_steps JSON")?,
        key_files: json_vec(row, "key_files", "invalid session summary key_files JSON")?,
        origin_context: json_opt(
            row,
            "origin_context",
            "invalid session summary origin_context JSON",
        )?,
        created_at: required_i64(row, "created_at")?,
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
        id: required_string(row, schema::ID)?,
        session_summary_id: required_string(row, schema::SESSION_SUMMARY_ID)?,
        agent_type,
        model: required_string(row, schema::MODEL)?,
        parent_session_id: row.try_get_string(schema::PARENT_SESSION_ID)?,
        started_at: required_i64(row, schema::STARTED_AT)?,
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
        id: required_string(row, "id")?,
        session_id: required_string(row, "session_id")?,
        checkpoint_type,
        description: required_string(row, "description")?,
        snapshot_data,
        created_at: required_i64(row, "created_at")?,
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
        created_at: required_i64(row, "created_at")?,
        updated_at: required_i64(row, "updated_at")?,
    })
}

/// Build a `ProjectIssue` from a port row.
pub fn row_to_issue(row: &dyn SqlRow) -> Result<ProjectIssue> {
    let labels_json = required_string(row, "labels")?;
    let labels = serde_json::from_str::<Vec<String>>(&labels_json)
        .map_err(|e| Error::memory_with_source("decode labels json", e))?;

    Ok(ProjectIssue {
        id: required_string(row, "id")?,
        org_id: required_string(row, "org_id")?,
        project_id: required_string(row, "project_id")?,
        created_by: required_string(row, "created_by")?,
        phase_id: row.try_get_string("phase_id")?,
        title: required_string(row, "title")?,
        description: required_string(row, "description")?,
        issue_type: IssueType::from_str(&required_string(row, "issue_type")?)
            .map_err(|e| Error::memory(e.to_string()))?,
        status: IssueStatus::from_str(&required_string(row, "status")?)
            .map_err(|e| Error::memory(e.to_string()))?,
        priority: required_i64(row, "priority")? as i32,
        assignee: row.try_get_string("assignee")?,
        labels,
        estimated_minutes: row.try_get_i64("estimated_minutes")?,
        actual_minutes: row.try_get_i64("actual_minutes")?,
        notes: required_string(row, "notes")?,
        design: required_string(row, "design")?,
        parent_issue_id: row.try_get_string("parent_issue_id")?,
        created_at: required_i64(row, "created_at")?,
        updated_at: required_i64(row, "updated_at")?,
        closed_at: row.try_get_i64("closed_at")?,
        closed_reason: required_string(row, "closed_reason")?,
    })
}

/// Build an `IssueComment` from a port row.
pub fn row_to_comment(row: &dyn SqlRow) -> Result<IssueComment> {
    Ok(IssueComment {
        id: required_string(row, "id")?,
        issue_id: required_string(row, "issue_id")?,
        author_id: required_string(row, "author_id")?,
        content: required_string(row, "content")?,
        created_at: required_i64(row, "created_at")?,
    })
}

/// Build an `IssueLabel` from a port row.
pub fn row_to_label(row: &dyn SqlRow) -> Result<IssueLabel> {
    Ok(IssueLabel {
        id: required_string(row, "id")?,
        org_id: required_string(row, "org_id")?,
        project_id: required_string(row, "project_id")?,
        name: required_string(row, "name")?,
        color: required_string(row, "color")?,
        created_at: required_i64(row, "created_at")?,
    })
}
