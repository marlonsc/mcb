use async_trait::async_trait;
use mcb_domain::{
    entities::memory::{
        MemoryFilter, MemorySearchResult, Observation, ObservationMetadata, ObservationType,
        SessionSummary,
    },
    error::{Error, Result},
    ports::MemoryRepository,
    ports::repositories::memory_repository::FtsSearchResult,
};

use sqlx::{Row, SqlitePool};
use std::path::PathBuf;
use tracing::{debug, info};

/// SQLite-based memory repository used by Phase 6 (Memory Search) to store observations, session
/// summaries, and FTS indexes so hybrid search pulls consistent data across configured vector stores.
pub struct SqliteMemoryRepository {
    pool: SqlitePool,
}

impl SqliteMemoryRepository {
    pub async fn new(db_path: PathBuf) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| Error::memory_with_source("Failed to create db directory", e))?;
        }

        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url)
            .await
            .map_err(|e| Error::memory_with_source("Failed to connect to SQLite", e))?;

        let repo = Self { pool };
        repo.init_schema()
            .await
            .map_err(|e| Error::memory_with_source("Failed to initialize memory schema", e))?;

        info!("Memory repository initialized at {}", db_path.display());
        Ok(repo)
    }

    pub async fn in_memory() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .map_err(|e| Error::memory_with_source("Failed to connect to in-memory SQLite", e))?;

        let repo = Self { pool };
        repo.init_schema()
            .await
            .map_err(|e| Error::memory_with_source("Failed to initialize in-memory schema", e))?;

        debug!("In-memory repository initialized");
        Ok(repo)
    }

    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS observations (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                content_hash TEXT UNIQUE NOT NULL,
                tags TEXT,
                observation_type TEXT,
                metadata TEXT,
                created_at INTEGER NOT NULL,
                embedding_id TEXT
            )
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create observations table", e))?;

        sqlx::query(
            r"
            CREATE TABLE IF NOT EXISTS session_summaries (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                topics TEXT,
                decisions TEXT,
                next_steps TEXT,
                key_files TEXT,
                created_at INTEGER NOT NULL
            )
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create session_summaries table", e))?;

        sqlx::query(
            "CREATE VIRTUAL TABLE IF NOT EXISTS observations_fts USING fts5(content, id UNINDEXED)",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create observations_fts table", e))?;

        sqlx::query(
            r"
            CREATE TRIGGER IF NOT EXISTS obs_ai AFTER INSERT ON observations BEGIN
              INSERT INTO observations_fts(id, content) VALUES (new.id, new.content);
            END;
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create obs_ai trigger", e))?;

        sqlx::query(
            r"
            CREATE TRIGGER IF NOT EXISTS obs_ad AFTER DELETE ON observations BEGIN
              DELETE FROM observations_fts WHERE id = old.id;
            END;
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create obs_ad trigger", e))?;

        sqlx::query(
            r"
            CREATE TRIGGER IF NOT EXISTS obs_au AFTER UPDATE ON observations BEGIN
              DELETE FROM observations_fts WHERE id = old.id;
              INSERT INTO observations_fts(id, content) VALUES (new.id, new.content);
            END;
            ",
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::memory_with_source("Failed to create obs_au trigger", e))?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_obs_hash ON observations(content_hash)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_obs_created ON observations(created_at)")
            .execute(&self.pool)
            .await
            .ok();

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_summary_session ON session_summaries(session_id)",
        )
        .execute(&self.pool)
        .await
        .ok();

        debug!("Memory schema initialized");
        Ok(())
    }

    fn row_to_observation(row: &sqlx::sqlite::SqliteRow) -> Result<Observation> {
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
            Some(r) => Ok(Some(Self::row_to_observation(&r).map_err(|e| {
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
            Some(r) => Ok(Some(Self::row_to_observation(&r).map_err(|e| {
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

        let mut results = Vec::new();
        for (i, row) in rows.iter().enumerate() {
            let observation = Self::row_to_observation(row)
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
                Self::row_to_observation(&row)
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
        let anchor = self.get_observation(anchor_id).await?;
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
                Self::row_to_observation(row)
                    .map_err(|e| Error::memory_with_source("Failed to decode observation", e))?,
            );
        }

        if let Some(anchor_obs) = self.get_observation(anchor_id).await? {
            timeline.push(anchor_obs);
        }

        for row in after_rows {
            timeline.push(
                Self::row_to_observation(&row)
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
            Some(r) => {
                let topics_json: Option<String> = r.try_get("topics").ok();
                let decisions_json: Option<String> = r.try_get("decisions").ok();
                let next_steps_json: Option<String> = r.try_get("next_steps").ok();
                let key_files_json: Option<String> = r.try_get("key_files").ok();

                Ok(Some(SessionSummary {
                    id: r
                        .try_get("id")
                        .map_err(|e| Error::memory_with_source("Missing id", e))?,
                    session_id: r
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
                    created_at: r
                        .try_get("created_at")
                        .map_err(|e| Error::memory_with_source("Missing created_at", e))?,
                }))
            }
            None => Ok(None),
        }
    }
}
