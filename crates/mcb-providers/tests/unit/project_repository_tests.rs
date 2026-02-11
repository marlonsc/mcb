use std::sync::Arc;

use mcb_domain::constants::keys::DEFAULT_ORG_ID;
use mcb_domain::entities::project::Project;
use mcb_domain::ports::infrastructure::{DatabaseExecutor, SqlParam};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_providers::database::{
    create_memory_repository_with_executor, create_project_repository_from_executor,
};

async fn setup_repository() -> (Arc<dyn ProjectRepository>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("Failed to create executor");
    seed_default_org(executor.as_ref()).await;
    let repo = create_project_repository_from_executor(executor);
    (repo, temp_dir)
}

async fn seed_default_org(executor: &dyn DatabaseExecutor) {
    executor
        .execute(
            "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
            &[
                SqlParam::String(DEFAULT_ORG_ID.to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("default".to_string()),
                SqlParam::String("{}".to_string()),
                SqlParam::I64(0),
                SqlParam::I64(0),
            ],
        )
        .await
        .expect("seed default org");
}

async fn setup_with_project(
    id: &str,
    name: &str,
    path: &str,
) -> (Arc<dyn ProjectRepository>, Project, tempfile::TempDir) {
    let (repo, temp_dir) = setup_repository().await;
    let project = create_test_project(id, name, path);
    repo.create(&project)
        .await
        .expect("Failed to create project");
    (repo, project, temp_dir)
}

fn create_test_project(id: &str, name: &str, path: &str) -> Project {
    let now = 1000000i64;
    Project {
        id: id.to_string(),
        org_id: DEFAULT_ORG_ID.to_string(),
        name: name.to_string(),
        path: path.to_string(),
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
async fn test_create_project() {
    let (repo, _project, _temp) = setup_with_project("proj-1", "Test Project", "/test/path").await;

    let retrieved = repo
        .get_by_id(DEFAULT_ORG_ID, "proj-1")
        .await
        .expect("Failed to get project");
    assert_eq!(retrieved.name, "Test Project");
}

#[tokio::test]
async fn test_get_project_by_id() {
    let (repo, project, _temp) = setup_with_project("proj-2", "Project 2", "/path/2").await;

    let p = repo
        .get_by_id(DEFAULT_ORG_ID, "proj-2")
        .await
        .expect("Failed to get project");
    assert_eq!(p.id, project.id);
    assert_eq!(p.org_id, DEFAULT_ORG_ID);
    assert_eq!(p.name, project.name);
    assert_eq!(p.path, project.path);
}

#[tokio::test]
async fn test_get_project_by_id_not_found() {
    let (repo, _temp) = setup_repository().await;

    let result = repo.get_by_id(DEFAULT_ORG_ID, "nonexistent").await;
    assert!(
        result.is_err(),
        "Expected not-found error for nonexistent project"
    );
}

#[tokio::test]
async fn test_get_project_by_name() {
    let (repo, _project, _temp) = setup_with_project("proj-3", "Unique Name", "/path/3").await;

    let retrieved = repo
        .get_by_name(DEFAULT_ORG_ID, "Unique Name")
        .await
        .expect("Failed to get project by name");
    assert_eq!(retrieved.id, "proj-3");
}

#[tokio::test]
async fn test_get_project_by_path() {
    let (repo, _project, _temp) = setup_with_project("proj-4", "Project 4", "/unique/path").await;

    let retrieved = repo
        .get_by_path(DEFAULT_ORG_ID, "/unique/path")
        .await
        .expect("Failed to get project by path");
    assert_eq!(retrieved.id, "proj-4");
}

#[tokio::test]
async fn test_list_projects() {
    let (repo, _temp) = setup_repository().await;
    let proj1 = create_test_project("proj-5", "Project 5", "/path/5");
    let proj2 = create_test_project("proj-6", "Project 6", "/path/6");

    repo.create(&proj1)
        .await
        .expect("Failed to create project 1");
    repo.create(&proj2)
        .await
        .expect("Failed to create project 2");

    let projects = repo
        .list(DEFAULT_ORG_ID)
        .await
        .expect("Failed to list projects");
    assert!(projects.len() >= 2);
    assert!(projects.iter().any(|p| p.id == "proj-5"));
    assert!(projects.iter().any(|p| p.id == "proj-6"));
}

#[tokio::test]
async fn test_update_project() {
    let (repo, _temp) = setup_repository().await;
    let mut project = create_test_project("proj-7", "Original Name", "/original/path");

    repo.create(&project)
        .await
        .expect("Failed to create project");

    project.name = "Updated Name".to_string();
    project.path = "/updated/path".to_string();
    project.updated_at = 2000000i64;

    repo.update(&project)
        .await
        .expect("Failed to update project");

    let p = repo
        .get_by_id(DEFAULT_ORG_ID, "proj-7")
        .await
        .expect("Failed to get project");
    assert_eq!(p.name, "Updated Name");
    assert_eq!(p.path, "/updated/path");
    assert_eq!(p.updated_at, 2000000i64);
}

#[tokio::test]
async fn test_delete_project() {
    let (repo, _project, _temp) = setup_with_project("proj-8", "To Delete", "/path/8").await;

    repo.delete(DEFAULT_ORG_ID, "proj-8")
        .await
        .expect("Failed to delete project");

    let result = repo.get_by_id(DEFAULT_ORG_ID, "proj-8").await;
    assert!(
        result.is_err(),
        "Expected not-found error for deleted project"
    );
}

#[tokio::test]
async fn test_org_isolation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");
    let (_mem_repo, executor) = create_memory_repository_with_executor(db_path)
        .await
        .expect("create executor");

    for org_id in &["org-A", "org-B"] {
        executor
            .execute(
                "INSERT OR IGNORE INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)",
                &[
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String(org_id.to_string()),
                    SqlParam::String("{}".to_string()),
                    SqlParam::I64(0),
                    SqlParam::I64(0),
                ],
            )
            .await
            .expect("seed org");
    }

    let repo = create_project_repository_from_executor(executor);
    let project = Project {
        id: "proj-iso".to_string(),
        org_id: "org-A".to_string(),
        name: "Org A Project".to_string(),
        path: "/orgA/path".to_string(),
        created_at: 1000000,
        updated_at: 1000000,
    };
    repo.create(&project).await.expect("create");

    assert!(repo.get_by_id("org-A", "proj-iso").await.is_ok());
    assert!(repo.get_by_id("org-B", "proj-iso").await.is_err());
    assert!(repo.list("org-B").await.unwrap().is_empty());
}
