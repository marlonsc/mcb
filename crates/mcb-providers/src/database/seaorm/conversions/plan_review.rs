//! Plan review entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan_review;
use mcb_domain::entities::plan::{PlanReview, ReviewVerdict};

crate::impl_conversion!(plan_review, PlanReview,
    direct: [id, org_id, plan_version_id, reviewer_id, feedback, created_at],
    enums: { verdict: ReviewVerdict = ReviewVerdict::NeedsRevision }
);
