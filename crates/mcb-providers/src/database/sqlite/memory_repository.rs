//! SQLite Memory Repository
// TODO(REF003): Missing test file for crates/mcb-providers/src/database/sqlite/memory_repository.rs.
// Expected: crates/mcb-providers/tests/memory_repository_test.rs
//!
//! # Overview
//! The `SqliteMemoryRepository` provides persistent storage for observations and session summaries
//! using a SQLite database. It serves as the primary source of truth for structured
//! memory data and enables full-text search (FTS) capabilities.
//!
//! # Responsibilities
//! - **Observation Persistence**: Storing and retrieving immutable observation records.
//! - **FTS Implementation**: Leveraging SQLite's FTS5 extension to perform efficient text searches.
//! - **Session Summaries**: Managing the persistence of high-level session insights.
//! - **Timeline Construction**: Querying observations efficiently by creation time.
//!
//! # Architecture
//! This repository implements `MemoryRepository` and sits in the Providers layer.
//! It depends on `DatabaseExecutor` (Port) to execute SQL, ensuring it remains
//! decoupled from the specific SQL client library (e.g., sqlx, rusqlite).

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::memory::{MemoryFilter, Observation, SessionSummary};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::memory_repository::{FtsSearchResult, MemoryRepository};
use mcb_domain::utils::mask_id;
use mcb_domain::value_objects::ids::{ObservationId, SessionId};
use tracing::debug;

use super::query_helpers;
use super::row_convert;

/// SQLite-based implementation of the `MemoryRepository`.
///
/// Uses standard SQL queries to manage `observations` and `session_summaries` tables.
/// Handles JSON serialization of complex fields (metadata, tags) and enforces
/// referential integrity via `ensure_parent` checks.
pub struct SqliteMemoryRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqliteMemoryRepository {
    /// Create a repository that uses the given executor (from provider factory).
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }
}

#[async_trait]
/// Persistent memory repository using SQLite.
impl MemoryRepository for SqliteMemoryRepository {
    /// Stores an observation record.
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        // Ensure default org and project exist
        super::ensure_parent::ensure_org_and_project(
            self.executor.as_ref(),
            &observation.project_id,
            observation.created_at,
        )
        .await?;

        let tags_json = serde_json::to_string(&observation.tags)
            .map_err(|e| Error::memory_with_source("serialize tags", e))?;
        let metadata_json = serde_json::to_string(&observation.metadata)
            .map_err(|e| Error::memory_with_source("serialize metadata", e))?;

        let params = [
            SqlParam::String(observation.id.clone()),
            SqlParam::String(observation.project_id.clone()),
            SqlParam::String(observation.content.clone()),
            SqlParam::String(observation.content_hash.clone()),
            SqlParam::String(tags_json),
            SqlParam::String(observation.r#type.as_str().to_owned()),
            SqlParam::String(metadata_json),
            SqlParam::I64(observation.created_at),
            observation
                .embedding_id
                .as_ref()
                .map_or(SqlParam::Null, |s| SqlParam::String(s.clone())),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO observations (id, project_id, content, content_hash, tags, observation_type, metadata, created_at, embedding_id)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(content_hash) DO UPDATE SET
                    tags = excluded.tags,
                    metadata = excluded.metadata
                ",
                &params,
            )
            .await?;

        debug!("Stored observation: {}", observation.id);
        Ok(())
    }

    /// Retrieves an observation by ID.
    // TODO(qlty): Found 16 lines of similar code in 2 locations (mass = 95)
    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM observations WHERE id = ?",
            &[SqlParam::String(id.as_str().to_owned())],
            row_convert::row_to_observation,
        )
        .await
    }

    /// Retrieves an observation by content hash.
    // TODO(qlty): Found 16 lines of similar code in 3 locations (mass = 91)
    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM observations WHERE content_hash = ?",
            &[SqlParam::String(content_hash.to_owned())],
            row_convert::row_to_observation,
        )
        .await
    }

    /// Searches observations using FTS.
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        let rows = self
            .executor
            .query_all(
                "SELECT id, rank FROM observations_fts WHERE observations_fts MATCH ? ORDER BY rank LIMIT ?",
                &[SqlParam::String(query.to_owned()), SqlParam::I64(limit as i64)],
            )
            .await?;

        let mut results = Vec::with_capacity(rows.len());
        for row in rows {
            let id = row
                .try_get_string("id")?
                .ok_or_else(|| Error::memory("FTS result missing id"))?;
            let rank = row.try_get_f64("rank")?.unwrap_or(0.0);
            results.push(FtsSearchResult { id, rank });
        }
        Ok(results)
    }

    /// Deletes an observation.
    async fn delete_observation(&self, id: &ObservationId) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM observations WHERE id = ?",
                &[SqlParam::String(id.as_str().to_owned())],
            )
            .await
    }

    /// Retrieves multiple observations by ID.
    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
        let sql = format!(
            "SELECT * FROM observations WHERE id IN ({})",
            placeholders.join(",")
        );
        let params: Vec<SqlParam> = ids
            .iter()
            .map(|id| SqlParam::String(id.as_str().to_owned()))
            .collect();

        query_helpers::query_all(
            &self.executor,
            &sql,
            &params,
            row_convert::row_to_observation,
            "observation",
        )
        .await
    }

    /// Retrieves a timeline of observations centered around an anchor observation.
    ///
    /// The timeline includes a specified number of observations before and after the anchor,
    /// optionally filtered by session, repository, or observation type.
    // TODO(KISS005): Function get_timeline is too long (76 lines, max: 50).
    // Break into smaller, focused functions for query building and result processing.
    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        let anchor = self.get_observation(anchor_id).await?;
        let anchor_time = match anchor {
            Some(obs) => obs.created_at,
            None => return Ok(Vec::new()),
        };

        let mut base_sql = String::from("SELECT * FROM observations WHERE 1=1");
        let mut base_params: Vec<SqlParam> = Vec::new();

        if let Some(ref f) = filter {
            if let Some(session_id) = &f.session_id {
                base_sql.push_str(" AND json_extract(metadata, '$.session_id') = ?");
                base_params.push(SqlParam::String(session_id.clone()));
            }
            if let Some(parent_session_id) = &f.parent_session_id {
                base_sql.push_str(
                    " AND json_extract(metadata, '$.origin_context.parent_session_id') = ?",
                );
                base_params.push(SqlParam::String(parent_session_id.clone()));
            }
            if let Some(repo_id) = &f.repo_id {
                base_sql.push_str(" AND json_extract(metadata, '$.repo_id') = ?");
                base_params.push(SqlParam::String(repo_id.clone()));
            }
            if let Some(branch) = &f.branch {
                base_sql.push_str(" AND json_extract(metadata, '$.branch') = ?");
                base_params.push(SqlParam::String(branch.clone()));
            }
            if let Some(commit) = &f.commit {
                base_sql.push_str(" AND json_extract(metadata, '$.commit') = ?");
                base_params.push(SqlParam::String(commit.clone()));
            }
            if let Some(obs_type) = &f.r#type {
                base_sql.push_str(" AND observation_type = ?");
                base_params.push(SqlParam::String(obs_type.as_str().to_owned()));
            }
        }

        let before_sql = format!(
            "{} AND created_at < ? ORDER BY created_at DESC LIMIT ?",
            base_sql
        );
        let mut before_params = base_params.clone();
        before_params.push(SqlParam::I64(anchor_time));
        before_params.push(SqlParam::I64(before as i64));

        let after_sql = format!(
            "{} AND created_at > ? ORDER BY created_at ASC LIMIT ?",
            base_sql
        );
        let mut after_params = base_params;
        after_params.push(SqlParam::I64(anchor_time));
        after_params.push(SqlParam::I64(after as i64));

        let before_rows = self.executor.query_all(&before_sql, &before_params).await?;
        let after_rows = self.executor.query_all(&after_sql, &after_params).await?;

        let mut timeline = Vec::new();
        for row in before_rows.iter().rev() {
            timeline.push(
                row_convert::row_to_observation(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode observation", e))?,
            );
        }
        if let Some(anchor_obs) = self.get_observation(anchor_id).await? {
            timeline.push(anchor_obs);
        }
        for row in after_rows {
            timeline.push(
                row_convert::row_to_observation(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode observation", e))?,
            );
        }
        Ok(timeline)
    }

    /// Persists a session summary to the database, updating it if it already exists.
    ///
    /// Handles serialization of topics, decisions, and other complex fields into JSON.
    // TODO(KISS005): Function store_session_summary is too long (54 lines, max: 50).
    // Break into smaller, focused functions for serialization and DB execution.
    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        // Ensure default org and project exist
        super::ensure_parent::ensure_org_and_project(
            self.executor.as_ref(),
            &summary.project_id,
            summary.created_at,
        )
        .await?;

        let topics_json = serde_json::to_string(&summary.topics)
            .map_err(|e| Error::memory_with_source("serialize topics", e))?;
        let decisions_json = serde_json::to_string(&summary.decisions)
            .map_err(|e| Error::memory_with_source("serialize decisions", e))?;
        let next_steps_json = serde_json::to_string(&summary.next_steps)
            .map_err(|e| Error::memory_with_source("serialize next_steps", e))?;
        let key_files_json = serde_json::to_string(&summary.key_files)
            .map_err(|e| Error::memory_with_source("serialize key_files", e))?;
        let origin_context_json = serde_json::to_string(&summary.origin_context)
            .map_err(|e| Error::memory_with_source("serialize origin_context", e))?;

        let params = [
            SqlParam::String(summary.id.clone()),
            SqlParam::String(summary.project_id.clone()),
            SqlParam::String(summary.session_id.clone()),
            SqlParam::String(topics_json),
            SqlParam::String(decisions_json),
            SqlParam::String(next_steps_json),
            SqlParam::String(key_files_json),
            SqlParam::String(origin_context_json),
            SqlParam::I64(summary.created_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO session_summaries (id, project_id, session_id, topics, decisions, next_steps, key_files, origin_context, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    topics = excluded.topics,
                    decisions = excluded.decisions,
                    next_steps = excluded.next_steps,
                    key_files = excluded.key_files,
                    origin_context = excluded.origin_context
                ",
                &params,
            )
            .await?;

        debug!(
            "Stored session summary for session: {}",
            mask_id(summary.session_id.as_str())
        );
        Ok(())
    }

    /// Retrieves the latest summary for a session.
    // TODO(qlty): Found 17 lines of similar code in 2 locations (mass = 95)
    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        query_helpers::query_one(
            &self.executor,
            "SELECT * FROM session_summaries WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
            &[SqlParam::String(session_id.as_str().to_owned())],
            row_convert::row_to_session_summary,
        )
        .await
    }
}
