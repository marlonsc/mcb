use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::plan::{Plan, PlanReview, PlanStatus, PlanVersion, ReviewVerdict};
use mcb_domain::ports::infrastructure::DatabaseExecutor;
use mcb_domain::ports::repositories::PlanEntityRepository;
use mcb_providers::database::SqlitePlanEntityRepository;

use super::entity_test_utils::{
    TEST_NOW, assert_not_found, seed_default_scope, seed_isolated_org_scope, setup_executor,
};

async fn setup_repo() -> (
    SqlitePlanEntityRepository,
    Arc<dyn DatabaseExecutor>,
    tempfile::TempDir,
) {
    let (executor, temp_dir) = setup_executor().await;
    seed_default_scope(executor.as_ref()).await;
    let repo = SqlitePlanEntityRepository::new(Arc::clone(&executor));
    (repo, executor, temp_dir)
}

fn create_test_plan(id: &str) -> Plan {
    Plan {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        project_id: "proj-1".to_string(),
        title: format!("Plan {id}"),
        description: format!("Description for plan {id}"),
        status: PlanStatus::Draft,
        created_by: "user-1".to_string(),
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
    }
}

fn create_test_version(id: &str, plan_id: &str, version_number: i64) -> PlanVersion {
    PlanVersion {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        plan_id: plan_id.to_string(),
        version_number,
        content_json: format!("{{\"v\": {version_number}}}"),
        change_summary: format!("Version {version_number} changes"),
        created_by: "user-1".to_string(),
        created_at: TEST_NOW,
    }
}

fn create_test_review(id: &str, plan_version_id: &str, verdict: ReviewVerdict) -> PlanReview {
    PlanReview {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        plan_version_id: plan_version_id.to_string(),
        reviewer_id: "user-1".to_string(),
        verdict,
        feedback: format!("Feedback for review {id}"),
        created_at: TEST_NOW,
    }
}

#[tokio::test]
async fn test_plan_crud() {
    let (repo, _executor, _temp) = setup_repo().await;
    let plan = create_test_plan("plan-1");

    repo.create_plan(&plan).await.expect("create");

    let retrieved = repo.get_plan(DEFAULT_ORG_ID, "plan-1").await.expect("get");
    assert_eq!(retrieved.title, "Plan plan-1");
    assert_eq!(retrieved.status, PlanStatus::Draft);

    let list = repo
        .list_plans(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("list");
    assert_eq!(list.len(), 1);

    let mut updated = plan.clone();
    updated.status = PlanStatus::Active;
    updated.updated_at = 2_000_000;
    repo.update_plan(&updated).await.expect("update");

    let after_update = repo.get_plan(DEFAULT_ORG_ID, "plan-1").await.expect("get");
    assert_eq!(after_update.status, PlanStatus::Active);

    repo.delete_plan(DEFAULT_ORG_ID, "plan-1")
        .await
        .expect("delete");
    assert_not_found(repo.get_plan(DEFAULT_ORG_ID, "plan-1").await);
}

#[tokio::test]
async fn test_plan_version_lifecycle() {
    let (repo, _executor, _temp) = setup_repo().await;
    let plan = create_test_plan("plan-1");
    repo.create_plan(&plan).await.expect("create plan");

    let v1 = create_test_version("v1", "plan-1", 1);
    let v2 = create_test_version("v2", "plan-1", 2);
    repo.create_plan_version(&v1).await.expect("create v1");
    repo.create_plan_version(&v2).await.expect("create v2");

    let retrieved = repo.get_plan_version("v1").await.expect("get");
    assert_eq!(retrieved.version_number, 1);

    let versions = repo
        .list_plan_versions_by_plan("plan-1")
        .await
        .expect("list");
    assert_eq!(versions.len(), 2);
    assert!(versions.iter().any(|v| v.version_number == 1));
    assert!(versions.iter().any(|v| v.version_number == 2));
}

#[tokio::test]
async fn test_plan_review_lifecycle() {
    let (repo, _executor, _temp) = setup_repo().await;
    let plan = create_test_plan("plan-1");
    repo.create_plan(&plan).await.expect("create plan");
    let v1 = create_test_version("v1", "plan-1", 1);
    repo.create_plan_version(&v1).await.expect("create v1");

    let r1 = create_test_review("r1", "v1", ReviewVerdict::NeedsRevision);
    let r2 = create_test_review("r2", "v1", ReviewVerdict::Approved);
    repo.create_plan_review(&r1).await.expect("create r1");
    repo.create_plan_review(&r2).await.expect("create r2");

    let retrieved = repo.get_plan_review("r1").await.expect("get");
    assert_eq!(retrieved.verdict, ReviewVerdict::NeedsRevision);

    let reviews = repo.list_plan_reviews_by_version("v1").await.expect("list");
    assert_eq!(reviews.len(), 2);
}

#[tokio::test]
async fn test_org_isolation_plans() {
    let (executor, _temp_dir) = setup_executor().await;

    for org_id in &["org-A", "org-B"] {
        seed_isolated_org_scope(executor.as_ref(), org_id).await;
    }

    let repo = SqlitePlanEntityRepository::new(executor);
    let plan = Plan {
        id: "plan-iso".to_string(),
        org_id: "org-A".to_string(),
        project_id: "proj-org-A".to_string(),
        title: "Org A Plan".to_string(),
        description: "belongs to A".to_string(),
        status: PlanStatus::Draft,
        created_by: "user-org-A".to_string(),
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
    };
    repo.create_plan(&plan).await.expect("create");

    assert!(repo.get_plan("org-A", "plan-iso").await.is_ok());
    assert_not_found(repo.get_plan("org-B", "plan-iso").await);
    assert!(
        repo.list_plans("org-B", "proj-org-B")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn test_plan_versioning_flow() {
    let (repo, _executor, _temp) = setup_repo().await;

    let plan = create_test_plan("plan-flow");
    repo.create_plan(&plan).await.expect("create plan");

    let v1 = create_test_version("v1", "plan-flow", 1);
    repo.create_plan_version(&v1).await.expect("create v1");

    let review_1 = create_test_review("r1", "v1", ReviewVerdict::NeedsRevision);
    repo.create_plan_review(&review_1)
        .await
        .expect("create review 1");

    let v2 = create_test_version("v2", "plan-flow", 2);
    repo.create_plan_version(&v2).await.expect("create v2");

    let review_2 = create_test_review("r2", "v2", ReviewVerdict::Approved);
    repo.create_plan_review(&review_2)
        .await
        .expect("create review 2");

    let mut updated_plan = plan.clone();
    updated_plan.status = PlanStatus::Active;
    updated_plan.updated_at = 2_000_000;
    repo.update_plan(&updated_plan).await.expect("update plan");

    let final_plan = repo
        .get_plan(DEFAULT_ORG_ID, "plan-flow")
        .await
        .expect("get");
    assert_eq!(final_plan.status, PlanStatus::Active);

    let versions = repo
        .list_plan_versions_by_plan("plan-flow")
        .await
        .expect("list versions");
    assert_eq!(versions.len(), 2);

    let reviews_v1 = repo
        .list_plan_reviews_by_version("v1")
        .await
        .expect("list reviews v1");
    assert_eq!(reviews_v1.len(), 1);
    assert_eq!(reviews_v1[0].verdict, ReviewVerdict::NeedsRevision);

    let reviews_v2 = repo
        .list_plan_reviews_by_version("v2")
        .await
        .expect("list reviews v2");
    assert_eq!(reviews_v2.len(), 1);
    assert_eq!(reviews_v2[0].verdict, ReviewVerdict::Approved);
}

#[tokio::test]
async fn test_delete_plan_with_versions_fails() {
    let (repo, _executor, _temp) = setup_repo().await;
    let plan = create_test_plan("plan-fk");
    repo.create_plan(&plan).await.expect("create plan");

    let v1 = create_test_version("v1", "plan-fk", 1);
    repo.create_plan_version(&v1).await.expect("create v1");

    let result = repo.delete_plan(DEFAULT_ORG_ID, "plan-fk").await;
    assert!(
        result.is_err(),
        "Deleting a plan with versions should fail due to FK constraint"
    );
}
