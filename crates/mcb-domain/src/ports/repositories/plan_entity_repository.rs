//! Plan Entity Repository Port
//!
//! # Overview
//! Defines the interface for persisting plan-related entities including plans,
//! versions, and reviews.
use async_trait::async_trait;

use crate::entities::plan::{Plan, PlanReview, PlanVersion};
use crate::error::Result;

#[async_trait]
/// Defines behavior for PlanEntityRepository.
#[async_trait]
/// Registry for plans.
pub trait PlanRegistry: Send + Sync {
    /// Performs the create plan operation.
    async fn create_plan(&self, plan: &Plan) -> Result<()>;
    /// Performs the get plan operation.
    async fn get_plan(&self, org_id: &str, id: &str) -> Result<Plan>;
    /// Performs the list plans operation.
    async fn list_plans(&self, org_id: &str, project_id: &str) -> Result<Vec<Plan>>;
    /// Performs the update plan operation.
    async fn update_plan(&self, plan: &Plan) -> Result<()>;
    /// Performs the delete plan operation.
    async fn delete_plan(&self, org_id: &str, id: &str) -> Result<()>;
}

#[async_trait]
/// Registry for plan versions.
pub trait PlanVersionRegistry: Send + Sync {
    /// Performs the create plan version operation.
    async fn create_plan_version(&self, version: &PlanVersion) -> Result<()>;
    /// Performs the get plan version operation.
    async fn get_plan_version(&self, id: &str) -> Result<PlanVersion>;
    /// Performs the list plan versions by plan operation.
    async fn list_plan_versions_by_plan(&self, plan_id: &str) -> Result<Vec<PlanVersion>>;
}

#[async_trait]
/// Registry for plan reviews.
pub trait PlanReviewRegistry: Send + Sync {
    /// Performs the create plan review operation.
    async fn create_plan_review(&self, review: &PlanReview) -> Result<()>;
    /// Performs the get plan review operation.
    async fn get_plan_review(&self, id: &str) -> Result<PlanReview>;
    /// Performs the list plan reviews by version operation.
    async fn list_plan_reviews_by_version(&self, plan_version_id: &str) -> Result<Vec<PlanReview>>;
}

/// Aggregate trait for plan entity management.
pub trait PlanEntityRepository:
    PlanRegistry + PlanVersionRegistry + PlanReviewRegistry + Send + Sync
{
}

impl<T> PlanEntityRepository for T where
    T: PlanRegistry + PlanVersionRegistry + PlanReviewRegistry + Send + Sync
{
}
