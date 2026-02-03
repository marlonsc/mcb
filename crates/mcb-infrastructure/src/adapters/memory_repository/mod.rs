//! SQLite-based memory repository for Phase 6 (Memory Search).
//!
//! This adapter implements the domain port `MemoryRepository` using a SQLite pool
//! supplied by `MemoryDatabaseProvider`. It does not open SQLite connections directly.

mod row_convert;

use async_trait::async_trait;
use mcb_domain::{
    entities::memory::{MemoryFilter, MemorySearchResult, Observation, SessionSummary},
    error::{Error, Result},
    ports::MemoryRepository,
    ports::repositories::memory_repository::FtsSearchResult,
};
use sqlx::{Row, SqlitePool};
use tracing::debug;

/// SQLite-based memory repository used by Phase 6 (Memory Search) to store observations, session
/// summaries, and FTS indexes so hybrid search pulls consistent data across configured vector stores.
///
/// The pool must be obtained from `MemoryDatabaseProvider`; the repository does not access SQLite
/// directly.
pub struct SqliteMemoryRepository {
    pool: SqlitePool,
}

impl SqliteMemoryRepository {
    /// Create a repository that uses the given pool (from `MemoryDatabaseProvider`).
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MemoryRepository for SqliteMemoryRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        let tags_json = serde_json::to_string(&observation.tags)
            .map_err(|e| Error::memory_with_source("Failed to serialize tags", e))?;
        let metadata_json = serde_json::to_string(&observation.metadata)
            .map_err(|e| Error::memory_with_source("Failed to serialize metadata", e))?;

        sqlx::query(
            r"
            INSERT INTO observations (id, content, content_hash, tags, observation_type, metadata, created_at, embedding_id)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(content_hash) DO UPDATE SET
                tags = excluded.tags,
                metadata = excluded.metadata
            ",
        )
        .bind(&observation.id)
        .bind(&observation.content)
        .bind(&observation.content_hash)
        .bind(&tags_json)
        .bind(observation.observation_type.as_str())
        .bind(&metadata_json)
        .bind(observation.created_at)
        .bind(&observation.embedding_id)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to store observation", e))?;

        debug!("Stored observation: {}", observation.id);
        Ok(())
    }

    async fn get_observation(&self, id: &str) -> Result<Option<Observation>> {
        let row = sqlx::query("SELECT * FROM observations WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to get observation", e))?;

        match row {
            Some(r) => Ok(Some(row_convert::row_to_observation(&r).map_err(|e| {
                Error::memory_with_source("Failed to decode observation row", e)
            })?)),
            None => Ok(None),
        }
    }

    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        let row = sqlx::query("SELECT * FROM observations WHERE content_hash = ?")
            .bind(content_hash)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to find by hash", e))?;

        match row {
            Some(r) => Ok(Some(row_convert::row_to_observation(&r).map_err(|e| {
                Error::memory_with_source("Failed to decode observation row", e)
            })?)),
            None => Ok(None),
        }
    }

    async fn search_fts(&self, query: &str, limit: usize) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT id FROM observations_fts WHERE observations_fts MATCH ? ORDER BY rank LIMIT ?",
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to search FTS", e))?;

        let mut ids = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| Error::memory_with_source("Failed to get id from FTS result", e))?;
            ids.push(id);
        }

        Ok(ids)
    }

    async fn search_fts_ranked(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        let rows = sqlx::query(
            "SELECT id, rank FROM observations_fts WHERE observations_fts MATCH ? ORDER BY rank LIMIT ?",
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to search FTS ranked", e))?;

        let mut results = Vec::new();
        for row in rows {
            let id: String = row
                .try_get("id")
                .map_err(|e| Error::memory_with_source("Failed to get id from FTS result", e))?;
            let rank: f64 = row
                .try_get("rank")
                .map_err(|e| Error::memory_with_source("Failed to get rank from FTS result", e))?;
            results.push(FtsSearchResult { id, rank });
        }

        Ok(results)
    }

    async fn delete_observation(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM observations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to delete observation", e))?;
        Ok(())
    }

    async fn search(
        &self,
        _query_embedding: &[f32],
        filter: MemoryFilter,
        limit: usize,
    ) -> Result<Vec<MemorySearchResult>> {
        let mut query = String::from("SELECT * FROM observations WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(session_id) = &filter.session_id {
            query.push_str(" AND json_extract(metadata, '$.session_id') = ?");
            params.push(session_id.clone());
        }

        if let Some(repo_id) = &filter.repo_id {
            query.push_str(" AND json_extract(metadata, '$.repo_id') = ?");
            params.push(repo_id.clone());
        }

        if let Some(branch) = &filter.branch {
            query.push_str(" AND json_extract(metadata, '$.branch') = ?");
            params.push(branch.clone());
        }

        if let Some(commit) = &filter.commit {
            query.push_str(" AND json_extract(metadata, '$.commit') = ?");
            params.push(commit.clone());
        }

        if let Some(obs_type) = &filter.observation_type {
            query.push_str(" AND observation_type = ?");
            params.push(obs_type.as_str().to_string());
        }

        if let Some((start, end)) = filter.time_range {
            query.push_str(" AND created_at >= ? AND created_at <= ?");
            params.push(start.to_string());
            params.push(end.to_string());
        }

        query.push_str(" ORDER BY created_at DESC LIMIT ?");
        params.push(limit.to_string());

        let mut q = sqlx::query(&query);
        for param in &params {
            q = q.bind(param);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to search observations", e))?;

        let mut results = Vec::with_capacity(rows.len());
        for (i, row) in rows.iter().enumerate() {
            let observation = row_convert::row_to_observation(row)
                .map_err(|e| Error::memory_with_source("Failed to decode observation row", e))?;
            results.push(MemorySearchResult {
                id: observation.id.clone(),
                observation,
                similarity_score: 1.0 - (i as f32 * 0.1).min(0.9),
            });
        }

        Ok(results)
    }

    async fn get_observations_by_ids(&self, ids: &[String]) -> Result<Vec<Observation>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let query = format!(
            "SELECT * FROM observations WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut q = sqlx::query(&query);
        for id in ids {
            q = q.bind(id);
        }

        let rows = q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to get observations by ids", e))?;

        let mut observations = Vec::new();
        for row in rows {
            observations.push(
                row_convert::row_to_observation(&row)
                    .map_err(|e| Error::memory_with_source("Failed to decode observation", e))?,
            );
        }

        Ok(observations)
    }

    async fn get_timeline(
        &self,
        anchor_id: &str,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        let anchor_res = self.get_observation(anchor_id).await;
        let anchor = anchor_res
            .map_err(|e| Error::memory_with_source("get_timeline: load anchor failed", e))?;
        let anchor_time = match anchor {
            Some(obs) => obs.created_at,
            None => return Ok(Vec::new()),
        };

        let mut base_query = String::from("SELECT * FROM observations WHERE 1=1");
        let mut params: Vec<String> = Vec::new();

        if let Some(ref f) = filter {
            if let Some(session_id) = &f.session_id {
                base_query.push_str(" AND json_extract(metadata, '$.session_id') = ?");
                params.push(session_id.clone());
            }
            if let Some(repo_id) = &f.repo_id {
                base_query.push_str(" AND json_extract(metadata, '$.repo_id') = ?");
                params.push(repo_id.clone());
            }
            if let Some(branch) = &f.branch {
                base_query.push_str(" AND json_extract(metadata, '$.branch') = ?");
                params.push(branch.clone());
            }
            if let Some(commit) = &f.commit {
                base_query.push_str(" AND json_extract(metadata, '$.commit') = ?");
                params.push(commit.clone());
            }
            if let Some(obs_type) = &f.observation_type {
                base_query.push_str(" AND observation_type = ?");
                params.push(obs_type.as_str().to_string());
            }
        }

        let before_query = format!(
            "{} AND created_at < ? ORDER BY created_at DESC LIMIT ?",
            base_query
        );
        let after_query = format!(
            "{} AND created_at > ? ORDER BY created_at ASC LIMIT ?",
            base_query
        );

        let mut before_q = sqlx::query(&before_query);
        for param in &params {
            before_q = before_q.bind(param);
        }
        before_q = before_q.bind(anchor_time).bind(before as i64);

        let before_rows = before_q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to get timeline before", e))?;

        let mut after_q = sqlx::query(&after_query);
        for param in &params {
            after_q = after_q.bind(param);
        }
        after_q = after_q.bind(anchor_time).bind(after as i64);

        let after_rows = after_q
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::memory_with_source("Failed to get timeline after", e))?;

        let mut timeline = Vec::new();

        for row in before_rows.iter().rev() {
            timeline.push(
                row_convert::row_to_observation(row)
                    .map_err(|e| Error::memory_with_source("Failed to decode observation", e))?,
            );
        }

        if let Some(anchor_obs) = self.get_observation(anchor_id).await? {
            timeline.push(anchor_obs);
        }

        for row in after_rows {
            timeline.push(
                row_convert::row_to_observation(&row)
                    .map_err(|e| Error::memory_with_source("Failed to decode observation", e))?,
            );
        }

        Ok(timeline)
    }

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        let topics_json = serde_json::to_string(&summary.topics)
            .map_err(|e| Error::memory_with_source("Failed to serialize topics", e))?;
        let decisions_json = serde_json::to_string(&summary.decisions)
            .map_err(|e| Error::memory_with_source("Failed to serialize decisions", e))?;
        let next_steps_json = serde_json::to_string(&summary.next_steps)
            .map_err(|e| Error::memory_with_source("Failed to serialize next_steps", e))?;
        let key_files_json = serde_json::to_string(&summary.key_files)
            .map_err(|e| Error::memory_with_source("Failed to serialize key_files", e))?;

        sqlx::query(
            r"
            INSERT INTO session_summaries (id, session_id, topics, decisions, next_steps, key_files, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                topics = excluded.topics,
                decisions = excluded.decisions,
                next_steps = excluded.next_steps,
                key_files = excluded.key_files
            ",
        )
        .bind(&summary.id)
        .bind(&summary.session_id)
        .bind(&topics_json)
        .bind(&decisions_json)
        .bind(&next_steps_json)
        .bind(&key_files_json)
        .bind(summary.created_at)
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to store session summary", e))?;

        debug!("Stored session summary for session: {}", summary.session_id);
        Ok(())
    }

    async fn get_session_summary(&self, session_id: &str) -> Result<Option<SessionSummary>> {
        let row = sqlx::query(
            "SELECT * FROM session_summaries WHERE session_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to get session summary", e))?;

        match row {
            Some(r) => Ok(Some(row_convert::row_to_session_summary(&r).map_err(
                |e| Error::memory_with_source("Failed to decode session summary row", e),
            )?)),
            None => Ok(None),
        }
    }
}
