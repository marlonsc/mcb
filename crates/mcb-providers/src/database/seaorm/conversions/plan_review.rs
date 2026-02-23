//! PlanReview domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::plan_review;
use mcb_domain::entities::plan::{PlanReview, ReviewVerdict};

impl From<plan_review::Model> for PlanReview {
    fn from(m: plan_review::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            plan_version_id: m.plan_version_id,
            reviewer_id: m.reviewer_id,
            verdict: m
                .verdict
                .parse::<ReviewVerdict>()
                .unwrap_or(ReviewVerdict::NeedsRevision),
            feedback: m.feedback,
            created_at: m.created_at,
        }
    }
}

impl From<PlanReview> for plan_review::ActiveModel {
    fn from(e: PlanReview) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            plan_version_id: ActiveValue::Set(e.plan_version_id),
            reviewer_id: ActiveValue::Set(e.reviewer_id),
            verdict: ActiveValue::Set(e.verdict.to_string()),
            feedback: ActiveValue::Set(e.feedback),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_plan_review() -> PlanReview {
        PlanReview {
            id: "pr-001".into(),
            org_id: "org-001".into(),
            plan_version_id: "pv-001".into(),
            reviewer_id: "usr-002".into(),
            verdict: ReviewVerdict::Approved,
            feedback: "Looks good!".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_plan_review() {
        let domain = sample_plan_review();
        let active: plan_review::ActiveModel = domain.clone().into();

        let model = plan_review::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            plan_version_id: active.plan_version_id.unwrap(),
            reviewer_id: active.reviewer_id.unwrap(),
            verdict: active.verdict.unwrap(),
            feedback: active.feedback.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: PlanReview = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.verdict, domain.verdict);
    }
}
