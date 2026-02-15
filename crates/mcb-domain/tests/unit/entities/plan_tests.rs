use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use rstest::rstest;

#[rstest]
#[case(PlanStatus::Draft, "draft")]
#[case(PlanStatus::Active, "active")]
#[case(PlanStatus::Executing, "executing")]
#[case(PlanStatus::Completed, "completed")]
#[case(PlanStatus::Archived, "archived")]
fn plan_status_as_str(#[case] status: PlanStatus, #[case] expected: &str) {
    assert_eq!(status.as_str(), expected);
}

#[rstest]
#[case("draft", Ok(PlanStatus::Draft))]
#[case("active", Ok(PlanStatus::Active))]
#[case("executing", Ok(PlanStatus::Executing))]
#[case("completed", Ok(PlanStatus::Completed))]
#[case("archived", Ok(PlanStatus::Archived))]
#[case("DRAFT", Ok(PlanStatus::Draft))]
#[case("Active", Ok(PlanStatus::Active))]
#[case("EXECUTING", Ok(PlanStatus::Executing))]
#[case("invalid", Err(()))]
fn plan_status_from_str(#[case] input: &str, #[case] expected: Result<PlanStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<PlanStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<PlanStatus>().is_err()),
    }
}

#[rstest]
#[case("plan")]
#[case("version")]
#[case("review")]
fn entity_construction(#[case] entity: &str) {
    match entity {
        "plan" => {
            let plan = Plan {
                metadata: mcb_domain::entities::EntityMetadata {
                    id: "plan-001".to_owned(),
                    created_at: 1000,
                    updated_at: 1000,
                },
                org_id: "org-001".to_owned(),
                project_id: "proj-001".to_owned(),
                title: "Migration plan".to_owned(),
                description: "Migrate schema and data".to_owned(),
                status: PlanStatus::Draft,
                created_by: "user-001".to_owned(),
            };
            assert_eq!(plan.metadata.id, "plan-001");
            assert_eq!(plan.org_id, "org-001");
            assert_eq!(plan.project_id, "proj-001");
            assert_eq!(plan.status, PlanStatus::Draft);
        }
        "version" => {
            let version = PlanVersion {
                id: "pv-001".to_owned(),
                org_id: "org-001".to_owned(),
                plan_id: "plan-001".to_owned(),
                version_number: 1,
                content_json: "{\"steps\":[\"a\",\"b\"]}".to_owned(),
                change_summary: "Initial draft".to_owned(),
                created_by: "user-001".to_owned(),
                created_at: 1000,
            };
            assert_eq!(version.id, "pv-001");
            assert_eq!(version.plan_id, "plan-001");
            assert_eq!(version.version_number, 1);
        }
        "review" => {
            let review = PlanReview {
                id: "pr-001".to_owned(),
                org_id: "org-001".to_owned(),
                plan_version_id: "pv-001".to_owned(),
                reviewer_id: "user-003".to_owned(),
                verdict: ReviewVerdict::Approved,
                feedback: "Looks good".to_owned(),
                created_at: 7000,
            };
            assert_eq!(review.id, "pr-001");
            assert_eq!(review.plan_version_id, "pv-001");
            assert_eq!(review.verdict, ReviewVerdict::Approved);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case("plan")]
#[case("version")]
#[case("review")]
fn entity_serialization_roundtrip(#[case] entity: &str) {
    match entity {
        "plan" => {
            let plan = Plan {
                metadata: mcb_domain::entities::EntityMetadata {
                    id: "plan-002".to_owned(),
                    created_at: 2000,
                    updated_at: 3000,
                },
                org_id: "org-001".to_owned(),
                project_id: "proj-001".to_owned(),
                title: "Execution plan".to_owned(),
                description: "Execute rollout".to_owned(),
                status: PlanStatus::Active,
                created_by: "user-001".to_owned(),
            };

            let json = serde_json::to_string(&plan).expect("serialize");
            let deserialized: Plan = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized.metadata.id, "plan-002");
            assert_eq!(deserialized.title, "Execution plan");
            assert_eq!(deserialized.metadata.updated_at, 3000);
        }
        "version" => {
            let version = PlanVersion {
                id: "pv-002".to_owned(),
                org_id: "org-001".to_owned(),
                plan_id: "plan-002".to_owned(),
                version_number: 2,
                content_json: "{\"milestones\":2}".to_owned(),
                change_summary: "Added milestones".to_owned(),
                created_by: "user-002".to_owned(),
                created_at: 5000,
            };

            let json = serde_json::to_string(&version).expect("serialize");
            let deserialized: PlanVersion = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized.id, "pv-002");
            assert_eq!(deserialized.version_number, 2);
            assert_eq!(deserialized.created_by, "user-002");
        }
        "review" => {
            let review = PlanReview {
                id: "pr-002".to_owned(),
                org_id: "org-001".to_owned(),
                plan_version_id: "pv-002".to_owned(),
                reviewer_id: "user-004".to_owned(),
                verdict: ReviewVerdict::NeedsRevision,
                feedback: "Please split into phases".to_owned(),
                created_at: 8000,
            };

            let json = serde_json::to_string(&review).expect("serialize");
            let deserialized: PlanReview = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(deserialized.id, "pr-002");
            assert_eq!(deserialized.feedback, "Please split into phases");
            assert_eq!(deserialized.created_at, 8000);
        }
        _ => unreachable!(),
    }
}

#[rstest]
#[case(ReviewVerdict::Approved, "approved")]
#[case(ReviewVerdict::Rejected, "rejected")]
#[case(ReviewVerdict::NeedsRevision, "needs_revision")]
fn review_verdict_as_str(#[case] verdict: ReviewVerdict, #[case] expected: &str) {
    assert_eq!(verdict.as_str(), expected);
}

#[rstest]
#[case("approved", Ok(ReviewVerdict::Approved))]
#[case("rejected", Ok(ReviewVerdict::Rejected))]
#[case("needs_revision", Ok(ReviewVerdict::NeedsRevision))]
#[case("APPROVED", Ok(ReviewVerdict::Approved))]
#[case("Rejected", Ok(ReviewVerdict::Rejected))]
#[case("NEEDS_REVISION", Ok(ReviewVerdict::NeedsRevision))]
#[case("invalid", Err(()))]
fn review_verdict_from_str(#[case] input: &str, #[case] expected: Result<ReviewVerdict, ()>) {
    match expected {
        Ok(verdict) => assert_eq!(input.parse::<ReviewVerdict>(), Ok(verdict)),
        Err(()) => assert!(input.parse::<ReviewVerdict>().is_err()),
    }
}
