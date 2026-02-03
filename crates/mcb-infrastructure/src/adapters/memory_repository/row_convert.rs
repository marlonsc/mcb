//! Row-to-entity conversion for the memory repository.

use mcb_domain::entities::memory::{
    Observation, ObservationMetadata, ObservationType, SessionSummary,
};
use mcb_domain::error::{Error, Result};
use sqlx::Row;

/// Build an `Observation` from a SQLite row.
pub fn row_to_observation(row: &sqlx::sqlite::SqliteRow) -> Result<Observation> {
    let tags_json: Option<String> = row.try_get("tags").ok();
    let tags: Vec<String> = tags_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let obs_type_str: String = row
        .try_get("observation_type")
        .unwrap_or_else(|_| "context".to_string());
    let observation_type = obs_type_str.parse().unwrap_or(ObservationType::Context);

    let metadata_json: Option<String> = row.try_get("metadata").ok();
    let metadata: ObservationMetadata = metadata_json
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    Ok(Observation {
        id: row
            .try_get("id")
            .map_err(|e| Error::memory_with_source("Missing id", e))?,
        content: row
            .try_get("content")
            .map_err(|e| Error::memory_with_source("Missing content", e))?,
        content_hash: row
            .try_get("content_hash")
            .map_err(|e| Error::memory_with_source("Missing content_hash", e))?,
        tags,
        observation_type,
        metadata,
        created_at: row
            .try_get("created_at")
            .map_err(|e| Error::memory_with_source("Missing created_at", e))?,
        embedding_id: row.try_get("embedding_id").ok(),
    })
}

/// Build a `SessionSummary` from a SQLite row.
pub fn row_to_session_summary(row: &sqlx::sqlite::SqliteRow) -> Result<SessionSummary> {
    let topics_json: Option<String> = row.try_get("topics").ok();
    let decisions_json: Option<String> = row.try_get("decisions").ok();
    let next_steps_json: Option<String> = row.try_get("next_steps").ok();
    let key_files_json: Option<String> = row.try_get("key_files").ok();

    Ok(SessionSummary {
        id: row
            .try_get("id")
            .map_err(|e| Error::memory_with_source("Missing id", e))?,
        session_id: row
            .try_get("session_id")
            .map_err(|e| Error::memory_with_source("Missing session_id", e))?,
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
            .try_get("created_at")
            .map_err(|e| Error::memory_with_source("Missing created_at", e))?,
    })
}
