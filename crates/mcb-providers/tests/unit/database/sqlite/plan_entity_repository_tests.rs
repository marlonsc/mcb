use rstest::rstest;
use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use mcb_domain::ports::DatabaseExecutor;
use mcb_domain::ports::{PlanRegistry, PlanReviewRegistry, PlanVersionRegistry};
use mcb_providers::database::SqlitePlanEntityRepository;

use crate::utils::entity::{
    TEST_NOW, TestResult, assert_not_found, seed_default_scope, seed_isolated_org_scope,
    setup_executor,
};

async fn setup_repo() -> TestResult<(
    SqlitePlanEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
)> {
    let (executor, temp_dir) = setup_executor().await?;
    seed_default_scope(executor.as_ref()).await?;
    let repo = SqlitePlanEntityRepository::new(Arc::clone(&executor));
    Ok((repo, executor, temp_dir))
}

fn create_test_plan(id: &str) -> Plan {
    Plan {
        metadata: mcb_domain::entities::EntityMetadata {
            id: id.to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        org_id: DEFAULT_ORG_ID.to_owned(),
        project_id: "proj-1".to_owned(),
        title: format!("Plan {id}"),
        description: format!("Description for plan {id}"),
        status: PlanStatus::Draft,
        created_by: "user-1".to_owned(),
    }
}

fn create_test_version(id: &str, plan_id: &str, version_number: i64) -> PlanVersion {
    PlanVersion {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        plan_id: plan_id.to_owned(),
        version_number,
        content_json: format!("{{\"v\": {version_number}}}"),
        change_summary: format!("Version {version_number} changes"),
        created_by: "user-1".to_owned(),
        created_at: TEST_NOW,
    }
}

fn create_test_review(id: &str, plan_version_id: &str, verdict: ReviewVerdict) -> PlanReview {
    PlanReview {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        plan_version_id: plan_version_id.to_owned(),
        reviewer_id: "user-1".to_owned(),
        verdict,
        feedback: format!("Feedback for review {id}"),
        created_at: TEST_NOW,
    }
}

#[tokio::test]
async fn test_plan_crud() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let plan = create_test_plan("plan-1");

    repo.create_plan(&plan).await?;

    let retrieved = repo.get_plan(DEFAULT_ORG_ID, "plan-1").await?;
    assert_eq!(retrieved.title, "Plan plan-1");
    assert_eq!(retrieved.status, PlanStatus::Draft);

    let list = repo.list_plans(DEFAULT_ORG_ID, "proj-1").await?;
    assert_eq!(list.len(), 1);

    let mut updated = plan.clone();
    updated.status = PlanStatus::Active;
    updated.metadata.updated_at = 2_000_000;
    repo.update_plan(&updated).await?;

    let after_update = repo.get_plan(DEFAULT_ORG_ID, "plan-1").await?;
    assert_eq!(after_update.status, PlanStatus::Active);

    repo.delete_plan(DEFAULT_ORG_ID, "plan-1").await?;
    assert_not_found(&repo.get_plan(DEFAULT_ORG_ID, "plan-1").await);
    Ok(())
}

#[tokio::test]
async fn test_plan_version_lifecycle() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let plan = create_test_plan("plan-1");
    repo.create_plan(&plan).await?;

    let v1 = create_test_version("v1", "plan-1", 1);
    let v2 = create_test_version("v2", "plan-1", 2);
    repo.create_plan_version(&v1).await?;
    repo.create_plan_version(&v2).await?;

    let retrieved = repo.get_plan_version("v1").await?;
    assert_eq!(retrieved.version_number, 1);

    let versions = repo.list_plan_versions_by_plan("plan-1").await?;
    assert_eq!(versions.len(), 2);
    assert!(versions.iter().any(|v| v.version_number == 1));
    assert!(versions.iter().any(|v| v.version_number == 2));
    Ok(())
}

#[tokio::test]
async fn test_plan_review_lifecycle() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let plan = create_test_plan("plan-1");
    repo.create_plan(&plan).await?;
    let v1 = create_test_version("v1", "plan-1", 1);
    repo.create_plan_version(&v1).await?;

    let r1 = create_test_review("r1", "v1", ReviewVerdict::NeedsRevision);
    let r2 = create_test_review("r2", "v1", ReviewVerdict::Approved);
    repo.create_plan_review(&r1).await?;
    repo.create_plan_review(&r2).await?;

    let retrieved = repo.get_plan_review("r1").await?;
    assert_eq!(retrieved.verdict, ReviewVerdict::NeedsRevision);

    let reviews = repo.list_plan_reviews_by_version("v1").await?;
    assert_eq!(reviews.len(), 2);
    Ok(())
}

#[rstest]
#[case("org-A", true)]
#[case("org-B", false)]
#[tokio::test]
async fn org_isolation_plans(#[case] org_id: &str, #[case] should_find: bool) -> TestResult {
    let (executor, _temp_dir) = setup_executor().await?;

    for org_id in &["org-A", "org-B"] {
        seed_isolated_org_scope(executor.as_ref(), org_id).await?;
    }

    let repo = SqlitePlanEntityRepository::new(executor);
    let plan = Plan {
        metadata: mcb_domain::entities::EntityMetadata {
            id: "plan-iso".to_owned(),
            created_at: TEST_NOW,
            updated_at: TEST_NOW,
        },
        org_id: "org-A".to_owned(),
        project_id: "proj-org-A".to_owned(),
        title: "Org A Plan".to_owned(),
        description: "belongs to A".to_owned(),
        status: PlanStatus::Draft,
        created_by: "user-org-A".to_owned(),
    };
    repo.create_plan(&plan).await?;

    let get_result = repo.get_plan(org_id, "plan-iso").await;
    if should_find {
        assert!(get_result.is_ok());
    } else {
        assert_not_found(&get_result);
        assert!(repo.list_plans("org-B", "proj-org-B").await?.is_empty());
    }
    Ok(())
}

#[tokio::test]
async fn test_plan_versioning_flow() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;

    let plan = create_test_plan("plan-flow");
    repo.create_plan(&plan).await?;

    let v1 = create_test_version("v1", "plan-flow", 1);
    repo.create_plan_version(&v1).await?;

    let review_1 = create_test_review("r1", "v1", ReviewVerdict::NeedsRevision);
    repo.create_plan_review(&review_1).await?;

    let v2 = create_test_version("v2", "plan-flow", 2);
    repo.create_plan_version(&v2).await?;

    let review_2 = create_test_review("r2", "v2", ReviewVerdict::Approved);
    repo.create_plan_review(&review_2).await?;

    let mut updated_plan = plan.clone();
    updated_plan.status = PlanStatus::Active;
    updated_plan.metadata.updated_at = 2_000_000;
    repo.update_plan(&updated_plan).await?;

    let final_plan = repo.get_plan(DEFAULT_ORG_ID, "plan-flow").await?;
    assert_eq!(final_plan.status, PlanStatus::Active);

    let versions = repo.list_plan_versions_by_plan("plan-flow").await?;
    assert_eq!(versions.len(), 2);

    let reviews_v1 = repo.list_plan_reviews_by_version("v1").await?;
    assert_eq!(reviews_v1.len(), 1);
    assert_eq!(reviews_v1[0].verdict, ReviewVerdict::NeedsRevision);

    let reviews_v2 = repo.list_plan_reviews_by_version("v2").await?;
    assert_eq!(reviews_v2.len(), 1);
    assert_eq!(reviews_v2[0].verdict, ReviewVerdict::Approved);
    Ok(())
}

#[tokio::test]
async fn test_delete_plan_with_versions_fails() -> TestResult {
    let (repo, _executor, _temp) = setup_repo().await?;
    let plan = create_test_plan("plan-fk");
    repo.create_plan(&plan).await?;

    let v1 = create_test_version("v1", "plan-fk", 1);
    repo.create_plan_version(&v1).await?;

    let result = repo.delete_plan(DEFAULT_ORG_ID, "plan-fk").await;
    assert!(
        result.is_err(),
        "Deleting a plan with versions should fail due to FK constraint"
    );
    Ok(())
}
