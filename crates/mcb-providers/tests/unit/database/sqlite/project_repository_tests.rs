use rstest::rstest;
use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::project::Project;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_providers::database::{
    create_memory_repository_with_executor, create_project_repository_from_executor,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

async fn setup_repository() -> TestResult<(Arc<dyn ProjectRepository>, tempfile::TempDir)> {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");

    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path).await?;
    seed_default_org(executor.as_ref()).await?;
    let repo = create_project_repository_from_executor(executor);
    Ok((repo, temp_dir))
}

async fn seed_default_org(executor: &dyn DatabaseExecutor) -> TestResult {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_owned()),
                SqlParam::String("default".to_owned()),
                SqlParam::String("default".to_owned()),
                SqlParam::String("{}".to_owned()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await?;
    Ok(())
}

async fn setup_with_project(
    id: &str,
    name: &str,
    path: &str,
) -> TestResult<(Arc<dyn ProjectRepository>, Project, tempfile::TempDir)> {
    let (repo, temp_dir) = setup_repository().await?;
    let project = create_test_project(id, name, path);
    repo.create(&project).await?;
    Ok((repo, project, temp_dir))
}

fn create_test_project(id: &str, name: &str, path: &str) -> Project {
    let now = 1000000i64;
    Project {
        id: id.to_owned(),
        org_id: DEFAULT_ORG_ID.to_owned(),
        name: name.to_owned(),
        path: path.to_owned(),
        created_at: now,
        updated_at: now,
    }
}

#[rstest]
#[case("proj-1", "Test Project", "/test/path")]
#[case("proj-2", "Project 2", "/path/2")]
#[tokio::test]
async fn get_project_by_id(#[case] id: &str, #[case] name: &str, #[case] path: &str) -> TestResult {
    let (repo, project, _temp) = setup_with_project(id, name, path).await?;

    let p = repo.get_by_id(DEFAULT_ORG_ID, id).await?;
    assert_eq!(p.id, project.id);
    assert_eq!(p.org_id, DEFAULT_ORG_ID);
    assert_eq!(p.name, project.name);
    assert_eq!(p.path, project.path);
    Ok(())
}

#[tokio::test]
async fn test_get_project_by_id_not_found() -> TestResult {
    let (repo, _temp) = setup_repository().await?;

    let result = repo.get_by_id(DEFAULT_ORG_ID, "nonexistent").await;
    assert!(
        result.is_err(),
        "Expected not-found error for nonexistent project"
    );
    Ok(())
}

#[rstest]
#[case("name", "proj-3", "Unique Name", "/path/3", "Unique Name")]
#[case("path", "proj-4", "Project 4", "/unique/path", "/unique/path")]
#[tokio::test]
async fn get_project_by_unique_fields(
    #[case] lookup_kind: &str,
    #[case] expected_id: &str,
    #[case] name: &str,
    #[case] path: &str,
    #[case] lookup_value: &str,
) -> TestResult {
    let (repo, _project, _temp) = setup_with_project(expected_id, name, path).await?;

    let retrieved = if lookup_kind == "name" {
        repo.get_by_name(DEFAULT_ORG_ID, lookup_value).await?
    } else {
        repo.get_by_path(DEFAULT_ORG_ID, lookup_value).await?
    };
    assert_eq!(retrieved.id, expected_id);
    Ok(())
}

#[tokio::test]
async fn test_list_projects() -> TestResult {
    let (repo, _temp) = setup_repository().await?;
    let proj1 = create_test_project("proj-5", "Project 5", "/path/5");
    let proj2 = create_test_project("proj-6", "Project 6", "/path/6");

    repo.create(&proj1).await?;
    repo.create(&proj2).await?;

    let projects = repo.list(DEFAULT_ORG_ID).await?;
    assert!(projects.len() >= 2);
    assert!(projects.iter().any(|p| p.id == "proj-5"));
    assert!(projects.iter().any(|p| p.id == "proj-6"));
    Ok(())
}

#[tokio::test]
async fn test_update_project() -> TestResult {
    let (repo, _temp) = setup_repository().await?;
    let mut project = create_test_project("proj-7", "Original Name", "/original/path");

    repo.create(&project).await?;

    project.name = "Updated Name".to_owned();
    project.path = "/updated/path".to_owned();
    project.updated_at = 2000000i64;

    repo.update(&project).await?;

    let p = repo.get_by_id(DEFAULT_ORG_ID, "proj-7").await?;
    assert_eq!(p.name, "Updated Name");
    assert_eq!(p.path, "/updated/path");
    assert_eq!(p.updated_at, 2000000i64);
    Ok(())
}

#[tokio::test]
async fn test_delete_project() -> TestResult {
    let (repo, _project, _temp) = setup_with_project("proj-8", "To Delete", "/path/8").await?;

    repo.delete(DEFAULT_ORG_ID, "proj-8").await?;

    let result = repo.get_by_id(DEFAULT_ORG_ID, "proj-8").await;
    assert!(
        result.is_err(),
        "Expected not-found error for deleted project"
    );
    Ok(())
}

#[tokio::test]
async fn test_org_isolation() -> TestResult {
    let temp_dir = tempfile::tempdir()?;
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path).await?;

    for org_id in &["org-A", "org-B"] {
        executor
            .execute(
                "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String("{}".to_owned()),
                    SqlParam::I64(0),
                    SqlParam::I64(0),
                ],
            )
            .await?;
    }

    let repo = create_project_repository_from_executor(executor);
    let project = Project {
        id: "proj-iso".to_owned(),
        org_id: "org-A".to_owned(),
        name: "Org A Project".to_owned(),
        path: "/orgA/path".to_owned(),
        created_at: 1000000,
        updated_at: 1000000,
    };
    repo.create(&project).await?;

    assert!(repo.get_by_id("org-A", "proj-iso").await.is_ok());
    assert!(repo.get_by_id("org-B", "proj-iso").await.is_err());
    assert!(repo.list("org-B").await?.is_empty());
    Ok(())
}
