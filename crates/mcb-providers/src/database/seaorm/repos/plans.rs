//! Plan registry implementations.
//!
//! Implements `PlanRegistry`, `PlanVersionRegistry`, and `PlanReviewRegistry`
//! for managing plans, versions, and reviews.

use super::*;

sea_impl_crud_scoped!(PlanRegistry for SeaOrmEntityRepository { db: db,
    entity: plan, domain: Plan, label: "Plan",
    scope_col: plan::Column::OrgId,
    create: create_plan(p),
    get: get_plan,
    list: list_plans(plan::Column::ProjectId => project_id),
    update: update_plan(p),
    delete: delete_plan
});

sea_impl_cgl!(PlanVersionRegistry for SeaOrmEntityRepository { db: db,
    entity: plan_version, domain: PlanVersion, label: "PlanVersion",
    create: create_plan_version(version),
    get: get_plan_version(id),
    list: list_plan_versions_by_plan(plan_version::Column::PlanId => plan_id),
});

sea_impl_cgl!(PlanReviewRegistry for SeaOrmEntityRepository { db: db,
    entity: plan_review, domain: PlanReview, label: "PlanReview",
    create: create_plan_review(review),
    get: get_plan_review(id),
    list: list_plan_reviews_by_version(plan_review::Column::PlanVersionId => plan_version_id),
});
