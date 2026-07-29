//! Plan repository ports.

use async_trait::async_trait;

use crate::entities::plan::{Plan, PlanReview, PlanVersion};

define_crud_port! {
    /// Registry for plans.
    pub trait PlanRegistry {
        entity: Plan,
        create: create_plan,
        get: get_plan(org_id, id),
        list: list_plans(org_id, project_id),
        update: update_plan,
        delete: delete_plan(org_id, id),
    }
}

define_readonly_port! {
    /// Registry for plan versions.
    pub trait PlanVersionRegistry {
        entity: PlanVersion,
        create: create_plan_version,
        get: get_plan_version(id),
        list: list_plan_versions_by_plan(plan_id),
    }
}

define_readonly_port! {
    /// Registry for plan reviews.
    pub trait PlanReviewRegistry {
        entity: PlanReview,
        create: create_plan_review,
        get: get_plan_review(id),
        list: list_plan_reviews_by_version(plan_version_id),
    }
}

define_aggregate! {
    /// Aggregate trait for plan entity management.
    #[async_trait]
    pub trait PlanEntityRepository = PlanRegistry + PlanVersionRegistry + PlanReviewRegistry;
}
