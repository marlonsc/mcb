//! SQLite memory repository using the domain port [`DatabaseExecutor`].
//!
//! Implements [`MemoryRepository`] via [`DatabaseExecutor`]; no direct sqlx in this module.

use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::memory::{MemoryFilter, Observation, SessionSummary};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::memory_repository::{FtsSearchResult, MemoryRepository};
use mcb_domain::value_objects::ids::{ObservationId, SessionId};
use mcb_infrastructure::logging::mask_id;
use tracing::debug;

use super::row_convert;

/// SQLite-based memory repository using the database executor port.
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
impl MemoryRepository for SqliteMemoryRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        // GAP-2: Auto-create project if missing to prevent FK constraint violation
        // This ensures the project exists before we try to link an observation to it.
        self.executor
            .execute(
                "INSERT OR IGNORE INTO projects (id, name, path, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(observation.project_id.clone()),
                    SqlParam::String(format!("Project {}", observation.project_id)),
                    SqlParam::String("default".to_string()),
                    SqlParam::I64(observation.created_at),
                    SqlParam::I64(observation.created_at),
                ],
            )
            .await
            .map_err(|e| Error::memory_with_source("auto-create project", e))?;

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
            SqlParam::String(observation.r#type.as_str().to_string()),
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

    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM observations WHERE id = ?",
                &[SqlParam::String(id.as_str().to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(row_convert::row_to_observation(r.as_ref()).map_err(
                |e| Error::memory_with_source("decode observation row", e),
            )?)),
            None => Ok(None),
        }
    }

    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM observations WHERE content_hash = ?",
                &[SqlParam::String(content_hash.to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(row_convert::row_to_observation(r.as_ref()).map_err(
                |e| Error::memory_with_source("decode observation row", e),
            )?)),
            None => Ok(None),
        }
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        let rows = self
            .executor
            .query_all(
                "SELECT id, rank FROM observations_fts WHERE observations_fts MATCH ? ORDER BY rank LIMIT ?",
                &[SqlParam::String(query.to_string()), SqlParam::I64(limit as i64)],
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

    async fn delete_observation(&self, id: &ObservationId) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM observations WHERE id = ?",
                &[SqlParam::String(id.as_str().to_string())],
            )
            .await
    }

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
            .map(|id| SqlParam::String(id.as_str().to_string()))
            .collect();

        let rows = self.executor.query_all(&sql, &params).await?;

        let mut observations = Vec::with_capacity(rows.len());
        for row in rows {
            observations.push(
                row_convert::row_to_observation(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode observation", e))?,
            );
        }
        Ok(observations)
    }

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
                base_params.push(SqlParam::String(obs_type.as_str().to_string()));
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

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        let topics_json = serde_json::to_string(&summary.topics)
            .map_err(|e| Error::memory_with_source("serialize topics", e))?;
        let decisions_json = serde_json::to_string(&summary.decisions)
            .map_err(|e| Error::memory_with_source("serialize decisions", e))?;
        let next_steps_json = serde_json::to_string(&summary.next_steps)
            .map_err(|e| Error::memory_with_source("serialize next_steps", e))?;
        let key_files_json = serde_json::to_string(&summary.key_files)
            .map_err(|e| Error::memory_with_source("serialize key_files", e))?;

        let params = [
            SqlParam::String(summary.id.clone()),
            SqlParam::String(summary.project_id.clone()),
            SqlParam::String(summary.session_id.clone()),
            SqlParam::String(topics_json),
            SqlParam::String(decisions_json),
            SqlParam::String(next_steps_json),
            SqlParam::String(key_files_json),
            SqlParam::I64(summary.created_at),
        ];

        self.executor
            .execute(
                r"
                INSERT INTO session_summaries (id, project_id, session_id, topics, decisions, next_steps, key_files, created_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO UPDATE SET
                    topics = excluded.topics,
                    decisions = excluded.decisions,
                    next_steps = excluded.next_steps,
                    key_files = excluded.key_files
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

    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        let row = self
            .executor
            .query_one(
                "SELECT * FROM session_summaries WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
                &[SqlParam::String(session_id.as_str().to_string())],
            )
            .await?;

        match row {
            Some(r) => Ok(Some(
                row_convert::row_to_session_summary(r.as_ref())
                    .map_err(|e| Error::memory_with_source("decode session summary row", e))?,
            )),
            None => Ok(None),
        }
    }
}
