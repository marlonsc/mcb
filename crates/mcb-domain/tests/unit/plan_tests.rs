use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};

#[test]
fn plan_status_as_str() {
    assert_eq!(PlanStatus::Draft.as_str(), "draft");
    assert_eq!(PlanStatus::Active.as_str(), "active");
    assert_eq!(PlanStatus::Executing.as_str(), "executing");
    assert_eq!(PlanStatus::Completed.as_str(), "completed");
    assert_eq!(PlanStatus::Archived.as_str(), "archived");
}

#[test]
fn plan_status_from_str() {
    assert_eq!("draft".parse::<PlanStatus>(), Ok(PlanStatus::Draft));
    assert_eq!("active".parse::<PlanStatus>(), Ok(PlanStatus::Active));
    assert_eq!("executing".parse::<PlanStatus>(), Ok(PlanStatus::Executing));
    assert_eq!("completed".parse::<PlanStatus>(), Ok(PlanStatus::Completed));
    assert_eq!("archived".parse::<PlanStatus>(), Ok(PlanStatus::Archived));
    assert!("invalid".parse::<PlanStatus>().is_err());
}

#[test]
fn plan_status_from_str_case_insensitive() {
    assert_eq!("DRAFT".parse::<PlanStatus>(), Ok(PlanStatus::Draft));
    assert_eq!("Active".parse::<PlanStatus>(), Ok(PlanStatus::Active));
    assert_eq!("EXECUTING".parse::<PlanStatus>(), Ok(PlanStatus::Executing));
}

#[test]
fn plan_construction() {
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
fn plan_serialization_roundtrip() {
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
fn plan_version_construction() {
    let version = PlanVersion {
        id: "pv-001".to_string(),
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
fn plan_version_serialization_roundtrip() {
    let version = PlanVersion {
        id: "pv-002".to_string(),
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

#[test]
fn review_verdict_as_str() {
    assert_eq!(ReviewVerdict::Approved.as_str(), "approved");
    assert_eq!(ReviewVerdict::Rejected.as_str(), "rejected");
    assert_eq!(ReviewVerdict::NeedsRevision.as_str(), "needs_revision");
}

#[test]
fn review_verdict_from_str() {
    assert_eq!(
        "approved".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::Approved)
    );
    assert_eq!(
        "rejected".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::Rejected)
    );
    assert_eq!(
        "needs_revision".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::NeedsRevision)
    );
    assert!("invalid".parse::<ReviewVerdict>().is_err());
}

#[test]
fn review_verdict_from_str_case_insensitive() {
    assert_eq!(
        "APPROVED".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::Approved)
    );
    assert_eq!(
        "Rejected".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::Rejected)
    );
    assert_eq!(
        "NEEDS_REVISION".parse::<ReviewVerdict>(),
        Ok(ReviewVerdict::NeedsRevision)
    );
}

#[test]
fn plan_review_construction() {
    let review = PlanReview {
        id: "pr-001".to_string(),
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
fn plan_review_serialization_roundtrip() {
    let review = PlanReview {
        id: "pr-002".to_string(),
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
