use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use rstest::rstest;

#[rstest]
#[case(PlanStatus::Draft, "draft")]
#[case(PlanStatus::Active, "active")]
#[case(PlanStatus::Executing, "executing")]
#[case(PlanStatus::Completed, "completed")]
#[case(PlanStatus::Archived, "archived")]
#[test]
fn test_plan_status_as_str(#[case] status: PlanStatus, #[case] expected: &str) {
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
#[test]
fn test_plan_status_from_str(#[case] input: &str, #[case] expected: Result<PlanStatus, ()>) {
    match expected {
        Ok(status) => assert_eq!(input.parse::<PlanStatus>(), Ok(status)),
        Err(()) => assert!(input.parse::<PlanStatus>().is_err()),
    }
}

#[test]
fn test_plan_construction() {
    let plan = Plan {
        id: "plan-001".to_string(),
        org_id: "org-001".to_string(),
        project_id: "proj-001".to_string(),
        title: "Migration plan".to_string(),
        description: "Migrate schema and data".to_string(),
        status: PlanStatus::Draft,
        created_by: "user-001".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };

    assert_eq!(plan.id, "plan-001");
    assert_eq!(plan.org_id, "org-001");
    assert_eq!(plan.project_id, "proj-001");
    assert_eq!(plan.status, PlanStatus::Draft);
}

#[test]
fn test_plan_serialization_roundtrip() {
    let plan = Plan {
        id: "plan-002".to_string(),
        org_id: "org-001".to_string(),
        project_id: "proj-001".to_string(),
        title: "Execution plan".to_string(),
        description: "Execute rollout".to_string(),
        status: PlanStatus::Active,
        created_by: "user-001".to_string(),
        created_at: 2000,
        updated_at: 3000,
    };

    let json = serde_json::to_string(&plan).expect("serialize");
    let deserialized: Plan = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "plan-002");
    assert_eq!(deserialized.title, "Execution plan");
    assert_eq!(deserialized.updated_at, 3000);
}

#[test]
fn test_plan_version_construction() {
    let version = PlanVersion {
        id: "pv-001".to_string(),
        org_id: "org-001".to_string(),
        plan_id: "plan-001".to_string(),
        version_number: 1,
        content_json: "{\"steps\":[\"a\",\"b\"]}".to_string(),
        change_summary: "Initial draft".to_string(),
        created_by: "user-001".to_string(),
        created_at: 1000,
    };

    assert_eq!(version.id, "pv-001");
    assert_eq!(version.plan_id, "plan-001");
    assert_eq!(version.version_number, 1);
}

#[test]
fn test_plan_version_serialization_roundtrip() {
    let version = PlanVersion {
        id: "pv-002".to_string(),
        org_id: "org-001".to_string(),
        plan_id: "plan-002".to_string(),
        version_number: 2,
        content_json: "{\"milestones\":2}".to_string(),
        change_summary: "Added milestones".to_string(),
        created_by: "user-002".to_string(),
        created_at: 5000,
    };

    let json = serde_json::to_string(&version).expect("serialize");
    let deserialized: PlanVersion = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "pv-002");
    assert_eq!(deserialized.version_number, 2);
    assert_eq!(deserialized.created_by, "user-002");
}

#[rstest]
#[case(ReviewVerdict::Approved, "approved")]
#[case(ReviewVerdict::Rejected, "rejected")]
#[case(ReviewVerdict::NeedsRevision, "needs_revision")]
#[test]
fn test_review_verdict_as_str(#[case] verdict: ReviewVerdict, #[case] expected: &str) {
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
#[test]
fn test_review_verdict_from_str(#[case] input: &str, #[case] expected: Result<ReviewVerdict, ()>) {
    match expected {
        Ok(verdict) => assert_eq!(input.parse::<ReviewVerdict>(), Ok(verdict)),
        Err(()) => assert!(input.parse::<ReviewVerdict>().is_err()),
    }
}

#[test]
fn test_plan_review_construction() {
    let review = PlanReview {
        id: "pr-001".to_string(),
        org_id: "org-001".to_string(),
        plan_version_id: "pv-001".to_string(),
        reviewer_id: "user-003".to_string(),
        verdict: ReviewVerdict::Approved,
        feedback: "Looks good".to_string(),
        created_at: 7000,
    };

    assert_eq!(review.id, "pr-001");
    assert_eq!(review.plan_version_id, "pv-001");
    assert_eq!(review.verdict, ReviewVerdict::Approved);
}

#[test]
fn test_plan_review_serialization_roundtrip() {
    let review = PlanReview {
        id: "pr-002".to_string(),
        org_id: "org-001".to_string(),
        plan_version_id: "pv-002".to_string(),
        reviewer_id: "user-004".to_string(),
        verdict: ReviewVerdict::NeedsRevision,
        feedback: "Please split into phases".to_string(),
        created_at: 8000,
    };

    let json = serde_json::to_string(&review).expect("serialize");
    let deserialized: PlanReview = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "pr-002");
    assert_eq!(deserialized.feedback, "Please split into phases");
    assert_eq!(deserialized.created_at, 8000);
}
