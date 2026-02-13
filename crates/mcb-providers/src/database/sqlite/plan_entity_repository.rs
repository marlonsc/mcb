use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::PlanEntityRepository;

use super::row_helpers::{req_i64, req_str};

/// SQLite-backed repository for plan, version, and review entities.
pub struct SqlitePlanEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqlitePlanEntityRepository {
    /// Creates a new repository using the provided database executor.
    // TODO(qlty): Found 31 lines of similar code in 3 locations (mass = 216)
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    /// Helper to query a single row and convert it.
    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

    /// Helper to query multiple rows and convert them.
    async fn query_all<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Vec<T>>
    where
        F: Fn(&dyn SqlRow) -> Result<T>,
    {
        let rows = self.executor.query_all(sql, params).await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(
                convert(row.as_ref())
                    .map_err(|e| Error::memory_with_source("decode plan entity", e))?,
            );
        }
        Ok(result)
    }
}

/// Converts a SQL row to a Plan.
fn row_to_plan(row: &dyn SqlRow) -> Result<Plan> {
    let status = req_str(row, "status")?
        .parse::<PlanStatus>()
        .map_err(|e| Error::memory(format!("Invalid plan status: {e}")))?;

    Ok(Plan {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        title: req_str(row, "title")?,
        description: req_str(row, "description")?,
        status,
        created_by: req_str(row, "created_by")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

/// Converts a SQL row to a PlanVersion.
fn row_to_plan_version(row: &dyn SqlRow) -> Result<PlanVersion> {
    Ok(PlanVersion {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        plan_id: req_str(row, "plan_id")?,
        version_number: req_i64(row, "version_number")?,
        content_json: req_str(row, "content_json")?,
        change_summary: req_str(row, "change_summary")?,
        created_by: req_str(row, "created_by")?,
        created_at: req_i64(row, "created_at")?,
    })
}

/// Converts a SQL row to a PlanReview.
fn row_to_plan_review(row: &dyn SqlRow) -> Result<PlanReview> {
    let verdict = req_str(row, "verdict")?
        .parse::<ReviewVerdict>()
        .map_err(|e| Error::memory(format!("Invalid review verdict: {e}")))?;

    Ok(PlanReview {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        plan_version_id: req_str(row, "plan_version_id")?,
        reviewer_id: req_str(row, "reviewer_id")?,
        verdict,
        feedback: req_str(row, "feedback")?,
        created_at: req_i64(row, "created_at")?,
    })
}

#[async_trait]
/// Persistent plan entity repository using SQLite.
impl PlanEntityRepository for SqlitePlanEntityRepository {
    /// Creates a new plan.
    async fn create_plan(&self, plan: &Plan) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO plans (id, org_id, project_id, title, description, status, created_by, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(plan.id.clone()),
                    SqlParam::String(plan.org_id.clone()),
                    SqlParam::String(plan.project_id.clone()),
                    SqlParam::String(plan.title.clone()),
                    SqlParam::String(plan.description.clone()),
                    SqlParam::String(plan.status.to_string()),
                    SqlParam::String(plan.created_by.clone()),
                    SqlParam::I64(plan.created_at),
                    SqlParam::I64(plan.updated_at),
                ],
            )
            .await
    }

    /// Retrieves a plan by org ID and plan ID.
    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan> {
        self.query_one(
            "SELECT * FROM plans WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(id.to_string()),
            ],
            row_to_plan,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("Plan {id}")))
    }

    /// Lists plans in an organization for a project.
    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>> {
        self.query_all(
            "SELECT * FROM plans WHERE org_id = ? AND project_id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(project_id.to_string()),
            ],
            row_to_plan,
        )
        .await
    }

    /// Updates an existing plan.
    async fn update_plan(&self, plan: &Plan) -> Result<()> {
        self.executor
            .execute(
                "UPDATE plans SET title = ?, description = ?, status = ?, updated_at = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(plan.title.clone()),
                    SqlParam::String(plan.description.clone()),
                    SqlParam::String(plan.status.to_string()),
                    SqlParam::I64(plan.updated_at),
                    SqlParam::String(plan.org_id.clone()),
                    SqlParam::String(plan.id.clone()),
                ],
            )
            .await
    }

    /// Deletes a plan.
    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()> {
        self.executor
            .execute(
                "DELETE FROM plans WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(id.to_string()),
                ],
            )
            .await
    }

    /// Creates a new plan version.
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO plan_versions (id, org_id, plan_id, version_number, content_json, change_summary, created_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(version.id.clone()),
                    SqlParam::String(version.org_id.clone()),
                    SqlParam::String(version.plan_id.clone()),
                    SqlParam::I64(version.version_number),
                    SqlParam::String(version.content_json.clone()),
                    SqlParam::String(version.change_summary.clone()),
                    SqlParam::String(version.created_by.clone()),
                    SqlParam::I64(version.created_at),
                ],
            )
            .await
    }

    /// Retrieves a plan version by ID.
    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion> {
        self.query_one(
            "SELECT * FROM plan_versions WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_plan_version,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("PlanVersion {id}")))
    }

    /// Lists versions of a plan.
    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        self.query_all(
            "SELECT * FROM plan_versions WHERE plan_id = ?",
            &[SqlParam::String(plan_id.to_string())],
            row_to_plan_version,
        )
        .await
    }

    /// Creates a new plan review.
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO plan_reviews (id, org_id, plan_version_id, reviewer_id, verdict, feedback, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(review.id.clone()),
                    SqlParam::String(review.org_id.clone()),
                    SqlParam::String(review.plan_version_id.clone()),
                    SqlParam::String(review.reviewer_id.clone()),
                    SqlParam::String(review.verdict.to_string()),
                    SqlParam::String(review.feedback.clone()),
                    SqlParam::I64(review.created_at),
                ],
            )
            .await
    }

    /// Retrieves a plan review by ID.
    async fn get_plan_review(&self, id: &str) -> Result<PlanReview> {
        self.query_one(
            "SELECT * FROM plan_reviews WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_plan_review,
        )
        .await?
        .ok_or_else(|| Error::not_found(format!("PlanReview {id}")))
    }

    /// Lists reviews for a plan version.
    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        self.query_all(
            "SELECT * FROM plan_reviews WHERE plan_version_id = ?",
            &[SqlParam::String(plan_version_id.to_string())],
            row_to_plan_review,
        )
        .await
    }
}
