use std::sync::Arc;

use async_trait::async_trait;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanVersion};
use mcb_domain::error::{Error, Result};
use mcb_domain::ports::infrastructure::database::{DatabaseExecutor, SqlParam, SqlRow};
use mcb_domain::ports::repositories::PlanEntityRepository;

pub struct SqlitePlanEntityRepository {
    executor: Arc<dyn DatabaseExecutor>,
}

impl SqlitePlanEntityRepository {
    pub fn new(executor: Arc<dyn DatabaseExecutor>) -> Self {
        Self { executor }
    }

    async fn query_one<T, F>(&self, sql: &str, params: &[SqlParam], convert: F) -> Result<Option<T>>
    where
        F: FnOnce(&dyn SqlRow) -> Result<T>,
    {
        match self.executor.query_one(sql, params).await? {
            Some(r) => Ok(Some(convert(r.as_ref())?)),
            None => Ok(None),
        }
    }

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

fn row_to_plan(row: &dyn SqlRow) -> Result<Plan> {
    Ok(Plan {
        id: req_str(row, "id")?,
        org_id: req_str(row, "org_id")?,
        project_id: req_str(row, "project_id")?,
        title: req_str(row, "title")?,
        description: req_str(row, "description")?,
        status: req_str(row, "status")?,
        created_by: req_str(row, "created_by")?,
        created_at: req_i64(row, "created_at")?,
        updated_at: req_i64(row, "updated_at")?,
    })
}

fn row_to_plan_version(row: &dyn SqlRow) -> Result<PlanVersion> {
    Ok(PlanVersion {
        id: req_str(row, "id")?,
        plan_id: req_str(row, "plan_id")?,
        version_number: req_i64(row, "version_number")?,
        content_json: req_str(row, "content_json")?,
        change_summary: req_str(row, "change_summary")?,
        created_by: req_str(row, "created_by")?,
        created_at: req_i64(row, "created_at")?,
    })
}

fn row_to_plan_review(row: &dyn SqlRow) -> Result<PlanReview> {
    Ok(PlanReview {
        id: req_str(row, "id")?,
        plan_version_id: req_str(row, "plan_version_id")?,
        reviewer_id: req_str(row, "reviewer_id")?,
        verdict: req_str(row, "verdict")?,
        feedback: req_str(row, "feedback")?,
        created_at: req_i64(row, "created_at")?,
    })
}

fn req_str(row: &dyn SqlRow, col: &str) -> Result<String> {
    row.try_get_string(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

fn req_i64(row: &dyn SqlRow, col: &str) -> Result<i64> {
    row.try_get_i64(col)?
        .ok_or_else(|| Error::memory(format!("Missing {col}")))
}

#[async_trait]
impl PlanEntityRepository for SqlitePlanEntityRepository {
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
                    SqlParam::String(plan.status.clone()),
                    SqlParam::String(plan.created_by.clone()),
                    SqlParam::I64(plan.created_at),
                    SqlParam::I64(plan.updated_at),
                ],
            )
            .await
    }

    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Option<Plan>> {
        self.query_one(
            "SELECT * FROM plans WHERE org_id = ? AND id = ?",
            &[
                SqlParam::String(org_id.to_string()),
                SqlParam::String(id.to_string()),
            ],
            row_to_plan,
        )
        .await
    }

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

    async fn update_plan(&self, plan: &Plan) -> Result<()> {
        self.executor
            .execute(
                "UPDATE plans SET title = ?, description = ?, status = ?, updated_at = ? WHERE org_id = ? AND id = ?",
                &[
                    SqlParam::String(plan.title.clone()),
                    SqlParam::String(plan.description.clone()),
                    SqlParam::String(plan.status.clone()),
                    SqlParam::I64(plan.updated_at),
                    SqlParam::String(plan.org_id.clone()),
                    SqlParam::String(plan.id.clone()),
                ],
            )
            .await
    }

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

    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO plan_versions (id, plan_id, version_number, content_json, change_summary, created_by, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(version.id.clone()),
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

    async fn get_plan_version(&self, id: &str) -> Result<Option<PlanVersion>> {
        self.query_one(
            "SELECT * FROM plan_versions WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_plan_version,
        )
        .await
    }

    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>> {
        self.query_all(
            "SELECT * FROM plan_versions WHERE plan_id = ?",
            &[SqlParam::String(plan_id.to_string())],
            row_to_plan_version,
        )
        .await
    }

    async fn create_plan_review(&self, review: &PlanReview) -> Result<()> {
        self.executor
            .execute(
                "INSERT INTO plan_reviews (id, plan_version_id, reviewer_id, verdict, feedback, created_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(review.id.clone()),
                    SqlParam::String(review.plan_version_id.clone()),
                    SqlParam::String(review.reviewer_id.clone()),
                    SqlParam::String(review.verdict.clone()),
                    SqlParam::String(review.feedback.clone()),
                    SqlParam::I64(review.created_at),
                ],
            )
            .await
    }

    async fn get_plan_review(&self, id: &str) -> Result<Option<PlanReview>> {
        self.query_one(
            "SELECT * FROM plan_reviews WHERE id = ?",
            &[SqlParam::String(id.to_string())],
            row_to_plan_review,
        )
        .await
    }

    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>> {
        self.query_all(
            "SELECT * FROM plan_reviews WHERE plan_version_id = ?",
            &[SqlParam::String(plan_version_id.to_string())],
            row_to_plan_review,
        )
        .await
    }
}
