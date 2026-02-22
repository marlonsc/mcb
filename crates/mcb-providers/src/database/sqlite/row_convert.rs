//!
//! **Documentation**: [docs/modules/providers.md](../../../../../docs/modules/providers.md#database)
//!
//! Row-to-entity conversion using the domain port [`SqlRow`].

use crate::utils::sqlite::row::{json_opt, json_vec, req_i64, req_parsed, req_str};
use mcb_domain::constants::keys as schema;
use mcb_domain::entities::agent::{AgentSession, Checkpoint, CheckpointType};
use mcb_domain::entities::issue::{IssueComment, IssueLabel};
use mcb_domain::entities::memory::{Observation, ObservationMetadata, SessionSummary};
use mcb_domain::entities::project::{Project, ProjectIssue};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::SqlRow;
use mcb_domain::schema::COL_OBSERVATION_TYPE;

/// Trait for converting a database row to an entity type.
#[allow(dead_code)]
pub trait FromRow: Sized {
    /// Convert a database row to an instance of this type.
    fn from_row(row: &dyn SqlRow) -> Result<Self>;
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
        id: req_str(row, schema::ID)?,
        project_id: req_str(row, "project_id")?,
        content: req_str(row, "content")?,
        content_hash: req_str(row, "content_hash")?,
        tags,
        r#type: observation_type,
        metadata,
        created_at: req_i64(row, schema::CREATED_AT)?,
        embedding_id: row.try_get_string("embedding_id")?,
    })
}

/// Build a `SessionSummary` from a port row.
pub fn row_to_session_summary(row: &dyn SqlRow) -> Result<SessionSummary> {
    Ok(SessionSummary {
        id: req_str(row, "id")?,
        project_id: req_str(row, "project_id")?,
        org_id: req_str(row, "org_id")?,
        session_id: req_str(row, "session_id")?,
        topics: json_vec(row, "topics", "invalid session summary topics JSON")?,
        decisions: json_vec(row, "decisions", "invalid session summary decisions JSON")?,
        next_steps: json_vec(row, "next_steps", "invalid session summary next_steps JSON")?,
        key_files: json_vec(row, "key_files", "invalid session summary key_files JSON")?,
        origin_context: json_opt(
            row,
            "origin_context",
            "invalid session summary origin_context JSON",
        )?,
        created_at: req_i64(row, "created_at")?,
    })
}

/// Build an `AgentSession` from a port row.
pub fn row_to_agent_session(row: &dyn SqlRow) -> Result<AgentSession> {
    let agent_type = req_parsed(row, "agent_type")?;
    let status = req_parsed(row, "status")?;

    Ok(AgentSession {
        id: req_str(row, schema::ID)?,
        session_summary_id: req_str(row, schema::SESSION_SUMMARY_ID)?,
        agent_type,
        model: req_str(row, schema::MODEL)?,
        parent_session_id: row.try_get_string(schema::PARENT_SESSION_ID)?,
        started_at: req_i64(row, schema::STARTED_AT)?,
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
    let checkpoint_type: CheckpointType = req_parsed(row, "checkpoint_type")?;

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
        id: req_str(row, "id")?,
        session_id: req_str(row, "session_id")?,
        checkpoint_type,
        description: req_str(row, "description")?,
        snapshot_data,
        created_at: req_i64(row, "created_at")?,
        restored_at: row.try_get_i64("restored_at")?,
        expired,
    })
}

/// Build a `Project` from a port row.
pub fn row_to_project(row: &dyn SqlRow) -> Result<Project> {
    Ok(Project {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        name: req_str(row, "name")?,
        path: req_str(row, "path")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

/// Build a `ProjectIssue` from a port row.
pub fn row_to_issue(row: &dyn SqlRow) -> Result<ProjectIssue> {
    let labels_json = req_str(row, "labels")?;
    let labels = serde_json::from_str::<Vec<String>>(&labels_json)
        .map_err(|e| Error::memory_with_source("decode labels json", e))?;

    Ok(ProjectIssue {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        created_by: req_str(row, "created_by")?,
        phase_id: row.try_get_string("phase_id")?,
        title: req_str(row, "title")?,
        description: req_str(row, "description")?,
        issue_type: req_parsed(row, "issue_type")?,
        status: req_parsed(row, "status")?,
        priority: req_i64(row, "priority")? as i32,
        assignee: row.try_get_string("assignee")?,
        labels,
        estimated_minutes: row.try_get_i64("estimated_minutes")?,
        actual_minutes: row.try_get_i64("actual_minutes")?,
        notes: req_str(row, "notes")?,
        design: req_str(row, "design")?,
        parent_issue_id: row.try_get_string("parent_issue_id")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
        closed_at: row.try_get_i64("closed_at")?,
        closed_reason: req_str(row, "closed_reason")?,
    })
}

/// Build an `IssueComment` from a port row.
pub fn row_to_comment(row: &dyn SqlRow) -> Result<IssueComment> {
    Ok(IssueComment {
        id: req_str(row, "id")?,
        issue_id: req_str(row, "issue_id")?,
        author_id: req_str(row, "author_id")?,
        content: req_str(row, "content")?,
        created_at: req_i64(row, "created_at")?,
    })
}

/// Build an `IssueLabel` from a port row.
pub fn row_to_label(row: &dyn SqlRow) -> Result<IssueLabel> {
    Ok(IssueLabel {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        name: req_str(row, "name")?,
        color: req_str(row, "color")?,
        created_at: req_i64(row, "created_at")?,
    })
}

impl FromRow for Observation {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_observation(row)
    }
}

impl FromRow for SessionSummary {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_session_summary(row)
    }
}

impl FromRow for AgentSession {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_agent_session(row)
    }
}

impl FromRow for Checkpoint {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_checkpoint(row)
    }
}

impl FromRow for Project {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_project(row)
    }
}

impl FromRow for ProjectIssue {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_issue(row)
    }
}

impl FromRow for IssueComment {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_comment(row)
    }
}

impl FromRow for IssueLabel {
    fn from_row(row: &dyn SqlRow) -> Result<Self> {
        row_to_label(row)
    }
}
