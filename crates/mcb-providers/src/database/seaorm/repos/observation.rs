#![allow(clippy::missing_errors_doc)]

use async_trait::async_trait;
use mcb_domain::constants::keys::{DEFAULT_ORG_ID, DEFAULT_ORG_NAME};
use mcb_domain::entities::memory::{MemoryFilter, Observation, SessionSummary};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::{FtsSearchResult, MemoryRepository};
use mcb_domain::value_objects::{ObservationId, SessionId};
use sea_orm::entity::prelude::*;
use sea_orm::sea_query::{Expr, ExprTrait, OnConflict, Order, Query};
use sea_orm::{
    ActiveValue::Set, ColumnTrait, ConnectionTrait, DatabaseConnection, EntityTrait, QueryFilter,
    QueryOrder, Statement, Value,
};

use crate::database::seaorm::entities::{observation, organization, project, session_summary};

pub struct SeaOrmObservationRepository {
    db: DatabaseConnection,
}

impl SeaOrmObservationRepository {
    #[must_use]
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn db_err(context: &str, error: DbErr) -> Error {
        Error::memory_with_source(context, error)
    }

    fn ignore_not_inserted<T>(
        result: std::result::Result<T, DbErr>,
    ) -> std::result::Result<(), DbErr> {
        match result {
            Ok(_) | Err(DbErr::RecordNotInserted) => Ok(()),
            Err(e) => Err(e),
        }
    }

    async fn ensure_org_and_project(&self, project_id: &str, timestamp: i64) -> Result<()> {
        let org = organization::ActiveModel {
            id: Set(DEFAULT_ORG_ID.to_owned()),
            name: Set(DEFAULT_ORG_NAME.to_owned()),
            slug: Set(DEFAULT_ORG_NAME.to_owned()),
            settings_json: Set("{}".to_owned()),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
        };

        Self::ignore_not_inserted(
            organization::Entity::insert(org)
                .on_conflict(
                    OnConflict::column(organization::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&self.db)
                .await,
        )
        .map_err(|e| Self::db_err("auto-create default org", e))?;

        let proj = project::ActiveModel {
            id: Set(project_id.to_owned()),
            org_id: Set(DEFAULT_ORG_ID.to_owned()),
            name: Set(format!("Project {project_id}")),
            path: Set(project_id.to_owned()),
            created_at: Set(timestamp),
            updated_at: Set(timestamp),
        };

        Self::ignore_not_inserted(
            project::Entity::insert(proj)
                .on_conflict(
                    OnConflict::column(project::Column::Id)
                        .do_nothing()
                        .to_owned(),
                )
                .exec(&self.db)
                .await,
        )
        .map_err(|e| Self::db_err("auto-create project", e))?;

        Ok(())
    }

    fn build_list_sql(&self, filter: Option<&MemoryFilter>, limit: usize) -> Statement {
        let mut query = Query::select();
        query
            .columns([
                observation::Column::Id,
                observation::Column::ProjectId,
                observation::Column::Content,
                observation::Column::ContentHash,
                observation::Column::Tags,
                observation::Column::ObservationType,
                observation::Column::Metadata,
                observation::Column::CreatedAt,
                observation::Column::EmbeddingId,
            ])
            .from(observation::Entity)
            .order_by(observation::Column::CreatedAt, Order::Desc)
            .limit(limit as u64);

        if let Some(f) = filter {
            if let Some(id) = &f.id {
                query.and_where(Expr::col(observation::Column::Id).eq(id));
            }
            if let Some(project_id) = &f.project_id {
                query.and_where(Expr::col(observation::Column::ProjectId).eq(project_id));
            }
            if let Some(obs_type) = &f.r#type {
                query.and_where(
                    Expr::col(observation::Column::ObservationType).eq(obs_type.as_str()),
                );
            }
            if let Some(session_id) = &f.session_id {
                query.and_where(Expr::cust_with_values(
                    "json_extract(metadata, '$.session_id') = ?",
                    vec![Value::from(session_id.clone())],
                ));
            }
            if let Some(parent_session_id) = &f.parent_session_id {
                query.and_where(Expr::cust_with_values(
                    "json_extract(metadata, '$.origin_context.parent_session_id') = ?",
                    vec![Value::from(parent_session_id.clone())],
                ));
            }
            if let Some(repo_id) = &f.repo_id {
                query.and_where(Expr::cust_with_values(
                    "json_extract(metadata, '$.repo_id') = ?",
                    vec![Value::from(repo_id.clone())],
                ));
            }
            if let Some((start, end)) = f.time_range {
                query.and_where(Expr::col(observation::Column::CreatedAt).gte(start));
                query.and_where(Expr::col(observation::Column::CreatedAt).lte(end));
            }
            if let Some(branch) = &f.branch {
                query.and_where(Expr::cust_with_values(
                    "json_extract(metadata, '$.branch') = ?",
                    vec![Value::from(branch.clone())],
                ));
            }
            if let Some(commit) = &f.commit {
                query.and_where(Expr::cust_with_values(
                    "json_extract(metadata, '$.commit') = ?",
                    vec![Value::from(commit.clone())],
                ));
            }
            if let Some(tags) = &f.tags {
                for tag in tags {
                    query.and_where(Expr::cust_with_values(
                        "EXISTS (SELECT 1 FROM json_each(tags) WHERE value = ?)",
                        vec![Value::from(tag.clone())],
                    ));
                }
            }
        }

        self.db.get_database_backend().build(&query)
    }

    async fn list_by_filter(
        &self,
        filter: Option<&MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<Observation>> {
        let rows = self
            .db
            .query_all_raw(self.build_list_sql(filter, limit))
            .await
            .map_err(|e| Self::db_err("list observations", e))?;

        rows.into_iter()
            .map(|row| {
                let model = observation::Model {
                    id: row.try_get("", "id")?,
                    project_id: row.try_get("", "project_id")?,
                    content: row.try_get("", "content")?,
                    content_hash: row.try_get("", "content_hash")?,
                    tags: row.try_get("", "tags")?,
                    observation_type: row.try_get("", "observation_type")?,
                    metadata: row.try_get("", "metadata")?,
                    created_at: row.try_get("", "created_at")?,
                    embedding_id: row.try_get("", "embedding_id")?,
                };
                Ok(model.into())
            })
            .collect::<std::result::Result<Vec<_>, DbErr>>()
            .map_err(|e| Self::db_err("decode observations", e))
    }

    pub async fn list_observations(
        &self,
        filter: Option<&MemoryFilter>,
        limit: usize,
    ) -> Result<Vec<Observation>> {
        self.list_by_filter(filter, limit).await
    }

    pub async fn inject_observations(
        &self,
        filter: Option<&MemoryFilter>,
        limit: usize,
        max_chars: usize,
    ) -> Result<Vec<Observation>> {
        let candidates = self.list_by_filter(filter, limit).await?;
        let mut selected = Vec::new();
        let mut total_chars = 0usize;

        for obs in candidates {
            total_chars += obs.content.len();
            if total_chars > max_chars {
                break;
            }
            selected.push(obs);
        }

        Ok(selected)
    }
}

#[async_trait]
impl MemoryRepository for SeaOrmObservationRepository {
    async fn store_observation(&self, observation: &Observation) -> Result<()> {
        self.ensure_org_and_project(&observation.project_id, observation.created_at)
            .await?;

        let active: observation::ActiveModel = observation.clone().into();

        observation::Entity::insert(active)
            .on_conflict(
                OnConflict::column(observation::Column::ContentHash)
                    .update_columns([observation::Column::Tags, observation::Column::Metadata])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| Self::db_err("store observation", e))?;

        Ok(())
    }

    async fn get_observation(&self, id: &ObservationId) -> Result<Option<Observation>> {
        observation::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map(|model| model.map(Into::into))
            .map_err(|e| Self::db_err("get observation", e))
    }

    async fn find_by_hash(&self, content_hash: &str) -> Result<Option<Observation>> {
        observation::Entity::find()
            .filter(observation::Column::ContentHash.eq(content_hash))
            .one(&self.db)
            .await
            .map(|model| model.map(Into::into))
            .map_err(|e| Self::db_err("find observation by hash", e))
    }

    async fn search(&self, query: &str, limit: usize) -> Result<Vec<FtsSearchResult>> {
        if query.trim().is_empty() {
            let observations = self.list_by_filter(None, limit).await?;
            return Ok(observations
                .into_iter()
                .map(|obs| FtsSearchResult {
                    id: obs.id,
                    rank: 0.0,
                })
                .collect());
        }

        let escaped = query.replace('\'', "''");
        let sql = format!(
            "SELECT id, bm25(observations_fts) AS rank FROM observations_fts WHERE observations_fts MATCH '{escaped}' ORDER BY bm25(observations_fts) LIMIT {limit}"
        );

        let rows = self
            .db
            .query_all_raw(Statement::from_string(self.db.get_database_backend(), sql))
            .await
            .map_err(|e| Self::db_err("search observations using FTS5", e))?;

        rows.into_iter()
            .map(|row| {
                Ok(FtsSearchResult {
                    id: row.try_get("", "id")?,
                    rank: row.try_get("", "rank")?,
                })
            })
            .collect::<std::result::Result<Vec<_>, DbErr>>()
            .map_err(|e| Self::db_err("decode FTS5 results", e))
    }

    async fn delete_observation(&self, id: &ObservationId) -> Result<()> {
        observation::Entity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|e| Self::db_err("delete observation", e))?;
        Ok(())
    }

    async fn get_observations_by_ids(&self, ids: &[ObservationId]) -> Result<Vec<Observation>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let id_values: Vec<String> = ids.iter().map(ToString::to_string).collect();

        observation::Entity::find()
            .filter(observation::Column::Id.is_in(id_values))
            .all(&self.db)
            .await
            .map(|models| models.into_iter().map(Into::into).collect())
            .map_err(|e| Self::db_err("get observations by ids", e))
    }

    async fn get_timeline(
        &self,
        anchor_id: &ObservationId,
        before: usize,
        after: usize,
        filter: Option<MemoryFilter>,
    ) -> Result<Vec<Observation>> {
        let Some(anchor) = self.get_observation(anchor_id).await? else {
            return Ok(Vec::new());
        };

        let mut before_filter = filter.clone().unwrap_or_default();
        before_filter.time_range = Some((i64::MIN, anchor.created_at - 1));

        let mut after_filter = filter.unwrap_or_default();
        after_filter.time_range = Some((anchor.created_at + 1, i64::MAX));

        let mut before_items = self.list_by_filter(Some(&before_filter), before).await?;
        before_items.sort_by_key(|obs| obs.created_at);

        let mut timeline = before_items;
        timeline.push(anchor);

        let mut after_items = self.list_by_filter(Some(&after_filter), after).await?;
        after_items.sort_by_key(|obs| obs.created_at);
        timeline.extend(after_items);

        Ok(timeline)
    }

    async fn store_session_summary(&self, summary: &SessionSummary) -> Result<()> {
        self.ensure_org_and_project(&summary.project_id, summary.created_at)
            .await?;

        let active: session_summary::ActiveModel = summary.clone().into();

        session_summary::Entity::insert(active)
            .on_conflict(
                OnConflict::column(session_summary::Column::Id)
                    .update_columns([
                        session_summary::Column::Topics,
                        session_summary::Column::Decisions,
                        session_summary::Column::NextSteps,
                        session_summary::Column::KeyFiles,
                        session_summary::Column::OriginContext,
                    ])
                    .to_owned(),
            )
            .exec(&self.db)
            .await
            .map_err(|e| Self::db_err("store session summary", e))?;

        Ok(())
    }

    async fn get_session_summary(&self, session_id: &SessionId) -> Result<Option<SessionSummary>> {
        session_summary::Entity::find()
            .filter(session_summary::Column::SessionId.eq(session_id.to_string()))
            .order_by_desc(session_summary::Column::CreatedAt)
            .one(&self.db)
            .await
            .map(|model| model.map(Into::into))
            .map_err(|e| Self::db_err("get session summary", e))
    }
}
