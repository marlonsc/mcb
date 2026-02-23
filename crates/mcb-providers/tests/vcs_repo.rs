use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use mcb_domain::ports::VcsEntityRepository;
use mcb_providers::database::seaorm::entities::{organization, project};
use mcb_providers::database::seaorm::migration::Migrator;
use mcb_providers::database::seaorm::vcs_repo::SeaOrmVcsEntityRepository;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;

const TEST_ORG_ID: &str = "org-test";
const TEST_PROJECT_ID: &str = "proj-test";
const TEST_NOW: i64 = 1_700_000_000;

async fn setup_repo() -> SeaOrmVcsEntityRepository {
    let db = Database::connect("sqlite::memory:")
        .await
        .expect("connect to in-memory sqlite");
    Migrator::up(&db, None)
        .await
        .expect("apply seaorm migrations");

    organization::ActiveModel {
        id: Set(TEST_ORG_ID.to_owned()),
        name: Set("Test Org".to_owned()),
        slug: Set("test-org".to_owned()),
        settings_json: Set("{}".to_owned()),
        created_at: Set(TEST_NOW),
        updated_at: Set(TEST_NOW),
    }
    .insert(&db)
    .await
    .expect("seed organization");

    project::ActiveModel {
        id: Set(TEST_PROJECT_ID.to_owned()),
        org_id: Set(TEST_ORG_ID.to_owned()),
        name: Set("Test Project".to_owned()),
        path: Set("/tmp/test-project".to_owned()),
        created_at: Set(TEST_NOW),
        updated_at: Set(TEST_NOW),
    }
    .insert(&db)
    .await
    .expect("seed project");

    SeaOrmVcsEntityRepository::new(db)
}

fn sample_repo(id: &str, project_id: &str) -> Repository {
    Repository {
        id: id.to_owned(),
        org_id: TEST_ORG_ID.to_owned(),
        project_id: project_id.to_owned(),
        name: format!("repo-{id}"),
        url: format!("https://example.com/{id}.git"),
        local_path: format!("/tmp/{id}"),
        vcs_type: VcsType::Git,
        created_at: TEST_NOW,
        updated_at: TEST_NOW,
    }
}

fn sample_branch(id: &str, repository_id: &str, name: &str, head_commit: &str) -> Branch {
    Branch {
        id: id.to_owned(),
        org_id: TEST_ORG_ID.to_owned(),
        repository_id: repository_id.to_owned(),
        name: name.to_owned(),
        is_default: name == "main",
        head_commit: head_commit.to_owned(),
        upstream: None,
        created_at: TEST_NOW,
    }
}

#[tokio::test]
async fn repository_crud_and_discovery_fix() -> mcb_domain::error::Result<()> {
    let repo = setup_repo().await;

    let first = sample_repo("repo-1", TEST_PROJECT_ID);
    repo.create_repository(&first).await?;

    let fetched = repo.get_repository(TEST_ORG_ID, "repo-1").await?;
    assert_eq!(fetched.name, "repo-repo-1");

    let list_project = repo.list_repositories(TEST_ORG_ID, TEST_PROJECT_ID).await?;
    assert_eq!(list_project.len(), 1);

    let discovery_list = repo.list_repositories(TEST_ORG_ID, "").await?;
    assert_eq!(discovery_list.len(), 1);
    assert_eq!(discovery_list[0].id, "repo-1");

    let mut updated = first.clone();
    updated.name = "repo-updated".to_owned();
    updated.updated_at += 1;
    repo.update_repository(&updated).await?;

    let after_update = repo.get_repository(TEST_ORG_ID, "repo-1").await?;
    assert_eq!(after_update.name, "repo-updated");

    repo.delete_repository(TEST_ORG_ID, "repo-1").await?;
    assert!(repo.get_repository(TEST_ORG_ID, "repo-1").await.is_err());
    Ok(())
}

#[tokio::test]
async fn index_compare_search_and_impact() -> mcb_domain::error::Result<()> {
    let repo = setup_repo().await;

    let mut indexed = sample_repo("repo-idx", TEST_PROJECT_ID);
    repo.index_repository(&indexed).await?;

    indexed.name = "repo-idx-updated".to_owned();
    indexed.updated_at += 1;
    repo.index_repository(&indexed).await?;
    let fetched = repo.get_repository(TEST_ORG_ID, "repo-idx").await?;
    assert_eq!(fetched.name, "repo-idx-updated");

    let main = sample_branch("branch-main", "repo-idx", "main", "abc123");
    let dev = sample_branch("branch-dev", "repo-idx", "develop", "def456");
    repo.create_branch(&main).await?;
    repo.create_branch(&dev).await?;

    let search_result = repo.search_branch("repo-idx", "dev", 10).await?;
    assert_eq!(search_result.len(), 1);
    assert_eq!(search_result[0].name, "develop");

    let compare = repo.compare_branches("repo-idx", "main", "develop").await?;
    assert_eq!(compare.base_head_commit.as_deref(), Some("abc123"));
    assert_eq!(compare.target_head_commit.as_deref(), Some("def456"));
    assert!(!compare.heads_equal);

    let impact = repo.analyze_impact("repo-idx", "main", "develop").await?;
    assert_eq!(impact.total_worktrees, 0);
    assert_eq!(impact.branch_worktree_counts.get("main").copied(), Some(0));
    assert_eq!(
        impact.branch_worktree_counts.get("develop").copied(),
        Some(0)
    );

    Ok(())
}
