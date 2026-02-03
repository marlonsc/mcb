//! Row-to-entity conversion using the domain port [`SqlRow`].

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
