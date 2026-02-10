//! Comprehensive unit tests for SqliteProjectRepository
//!
//! Tests cover all CRUD operations and filtering for projects, phases, issues,
//! dependencies, and decisions.

use std::sync::Arc;

use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, Project, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_providers::database::create_project_repository;

// ============================================================================
// Helper Functions
// ============================================================================

async fn setup_repository() -> (Arc<dyn ProjectRepository>, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test.db");

    let repo = create_project_repository(db_path)
        .await
        .expect("Failed to create project repository");
    (repo, temp_dir)
}

/// Helper: Setup repository and create a test project, returning both
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
        name: name.to_string(),
        path: path.to_string(),
        created_at: now,
        updated_at: now,
    }
}

fn create_test_phase(id: &str, project_id: &str, name: &str, sequence: i32) -> ProjectPhase {
    let now = 1000000i64;
    ProjectPhase {
        id: id.to_string(),
        project_id: project_id.to_string(),
        name: name.to_string(),
        description: format!("Phase: {}", name),
        sequence,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn create_test_issue(
    id: &str,
    project_id: &str,
    title: &str,
    phase_id: Option<String>,
) -> ProjectIssue {
    let now = 1000000i64;
    ProjectIssue {
        id: id.to_string(),
        project_id: project_id.to_string(),
        phase_id,
        title: title.to_string(),
        description: format!("Issue: {}", title),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec!["test".to_string()],
        created_at: now,
        updated_at: now,
        closed_at: None,
    }
}

fn create_test_dependency(id: &str, from_issue_id: &str, to_issue_id: &str) -> ProjectDependency {
    let now = 1000000i64;
    ProjectDependency {
        id: id.to_string(),
        from_issue_id: from_issue_id.to_string(),
        to_issue_id: to_issue_id.to_string(),
        dependency_type: DependencyType::Blocks,
        created_at: now,
    }
}

fn create_test_decision(
    id: &str,
    project_id: &str,
    title: &str,
    issue_id: Option<String>,
) -> ProjectDecision {
    let now = 1000000i64;
    ProjectDecision {
        id: id.to_string(),
        project_id: project_id.to_string(),
        issue_id,
        title: title.to_string(),
        context: "Test context".to_string(),
        decision: "Test decision".to_string(),
        consequences: "Test consequences".to_string(),
        created_at: now,
    }
}

// ============================================================================
// Project CRUD Tests
// ============================================================================

#[tokio::test]
async fn test_create_project() {
    let (_repo, _project, _temp) = setup_with_project("proj-1", "Test Project", "/test/path").await;

    let retrieved = _repo
        .get_by_id("proj-1")
        .await
        .expect("Failed to get project");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test Project");
}

#[tokio::test]
async fn test_get_project_by_id() {
    let (repo, project, _temp) = setup_with_project("proj-2", "Project 2", "/path/2").await;

    let retrieved = repo
        .get_by_id("proj-2")
        .await
        .expect("Failed to get project");
    assert!(retrieved.is_some());
    let p = retrieved.unwrap();
    assert_eq!(p.id, project.id);
    assert_eq!(p.name, project.name);
    assert_eq!(p.path, project.path);
}

#[tokio::test]
async fn test_get_project_by_id_not_found() {
    let (repo, _temp) = setup_repository().await;

    let retrieved = repo
        .get_by_id("nonexistent")
        .await
        .expect("Failed to query");
    assert!(retrieved.is_none());
}

#[tokio::test]
async fn test_get_project_by_name() {
    let (repo, _project, _temp) = setup_with_project("proj-3", "Unique Name", "/path/3").await;

    let retrieved = repo
        .get_by_name("Unique Name")
        .await
        .expect("Failed to get project by name");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "proj-3");
}

#[tokio::test]
async fn test_get_project_by_path() {
    let (repo, _project, _temp) = setup_with_project("proj-4", "Project 4", "/unique/path").await;

    let retrieved = repo
        .get_by_path("/unique/path")
        .await
        .expect("Failed to get project by path");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().id, "proj-4");
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

    let projects = repo.list().await.expect("Failed to list projects");
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

    let retrieved = repo
        .get_by_id("proj-7")
        .await
        .expect("Failed to get project");
    assert!(retrieved.is_some());
    let p = retrieved.unwrap();
    assert_eq!(p.name, "Updated Name");
    assert_eq!(p.path, "/updated/path");
    assert_eq!(p.updated_at, 2000000i64);
}

#[tokio::test]
async fn test_delete_project() {
    let (repo, _project, _temp) = setup_with_project("proj-8", "To Delete", "/path/8").await;

    repo.delete("proj-8")
        .await
        .expect("Failed to delete project");

    let retrieved = repo.get_by_id("proj-8").await.expect("Failed to query");
    assert!(retrieved.is_none());
}

// ============================================================================
// Phase Operations Tests
// ============================================================================

#[tokio::test]
async fn test_create_phase() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-9", "Project 9", "/path/9");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let phase = create_test_phase("phase-1", "proj-9", "Phase 1", 1);
    repo.create_phase(&phase)
        .await
        .expect("Failed to create phase");

    let retrieved = repo
        .get_phase("phase-1")
        .await
        .expect("Failed to get phase");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Phase 1");
}

#[tokio::test]
async fn test_list_phases() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-10", "Project 10", "/path/10");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let phase1 = create_test_phase("phase-2", "proj-10", "Phase 1", 1);
    let phase2 = create_test_phase("phase-3", "proj-10", "Phase 2", 2);

    repo.create_phase(&phase1)
        .await
        .expect("Failed to create phase 1");
    repo.create_phase(&phase2)
        .await
        .expect("Failed to create phase 2");

    let phases = repo
        .list_phases("proj-10")
        .await
        .expect("Failed to list phases");
    assert_eq!(phases.len(), 2);
    assert_eq!(phases[0].sequence, 1);
    assert_eq!(phases[1].sequence, 2);
}

#[tokio::test]
async fn test_update_phase() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-11", "Project 11", "/path/11");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let mut phase = create_test_phase("phase-4", "proj-11", "Original Phase", 1);
    repo.create_phase(&phase)
        .await
        .expect("Failed to create phase");

    phase.name = "Updated Phase".to_string();
    phase.status = PhaseStatus::InProgress;
    phase.started_at = Some(1500000i64);
    phase.updated_at = 2000000i64;

    repo.update_phase(&phase)
        .await
        .expect("Failed to update phase");

    let retrieved = repo
        .get_phase("phase-4")
        .await
        .expect("Failed to get phase");
    assert!(retrieved.is_some());
    let p = retrieved.unwrap();
    assert_eq!(p.name, "Updated Phase");
    assert_eq!(p.status, PhaseStatus::InProgress);
    assert_eq!(p.started_at, Some(1500000i64));
}

// ============================================================================
// Issue Operations Tests
// ============================================================================

#[tokio::test]
async fn test_create_issue() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-12", "Project 12", "/path/12");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue = create_test_issue("issue-1", "proj-12", "Test Issue", None);
    repo.create_issue(&issue)
        .await
        .expect("Failed to create issue");

    let retrieved = repo
        .get_issue("issue-1")
        .await
        .expect("Failed to get issue");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Issue");
}

#[tokio::test]
async fn test_list_issues() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-13", "Project 13", "/path/13");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = create_test_issue("issue-2", "proj-13", "Issue 1", None);
    let issue2 = create_test_issue("issue-3", "proj-13", "Issue 2", None);

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let issues = repo
        .list_issues("proj-13")
        .await
        .expect("Failed to list issues");
    assert!(issues.len() >= 2);
    assert!(issues.iter().any(|i| i.id == "issue-2"));
    assert!(issues.iter().any(|i| i.id == "issue-3"));
}

#[tokio::test]
async fn test_filter_issues_by_status() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-14", "Project 14", "/path/14");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let mut issue1 = create_test_issue("issue-4", "proj-14", "Open Issue", None);
    let mut issue2 = create_test_issue("issue-5", "proj-14", "Closed Issue", None);

    issue1.status = IssueStatus::Open;
    issue2.status = IssueStatus::Closed;

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let filter = IssueFilter {
        project_id: Some("proj-14".to_string()),
        status: Some(IssueStatus::Open),
        ..Default::default()
    };

    let filtered = repo
        .filter_issues(&filter)
        .await
        .expect("Failed to filter issues");
    assert!(filtered.iter().all(|i| i.status == IssueStatus::Open));
    assert!(filtered.iter().any(|i| i.id == "issue-4"));
}

#[tokio::test]
async fn test_filter_issues_by_phase() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-15", "Project 15", "/path/15");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let phase = create_test_phase("phase-5", "proj-15", "Phase 1", 1);
    repo.create_phase(&phase)
        .await
        .expect("Failed to create phase");

    let issue1 = create_test_issue(
        "issue-6",
        "proj-15",
        "Issue in Phase",
        Some("phase-5".to_string()),
    );
    let issue2 = create_test_issue("issue-7", "proj-15", "Issue without Phase", None);

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let filter = IssueFilter {
        project_id: Some("proj-15".to_string()),
        phase_id: Some("phase-5".to_string()),
        ..Default::default()
    };

    let filtered = repo
        .filter_issues(&filter)
        .await
        .expect("Failed to filter issues");
    assert!(
        filtered
            .iter()
            .all(|i| i.phase_id == Some("phase-5".to_string()))
    );
    assert!(filtered.iter().any(|i| i.id == "issue-6"));
}

#[tokio::test]
async fn test_update_issue() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-16", "Project 16", "/path/16");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let mut issue = create_test_issue("issue-8", "proj-16", "Original Title", None);
    repo.create_issue(&issue)
        .await
        .expect("Failed to create issue");

    issue.title = "Updated Title".to_string();
    issue.status = IssueStatus::InProgress;
    issue.priority = 1;
    issue.assignee = Some("user@example.com".to_string());
    issue.updated_at = 2000000i64;

    repo.update_issue(&issue)
        .await
        .expect("Failed to update issue");

    let retrieved = repo
        .get_issue("issue-8")
        .await
        .expect("Failed to get issue");
    assert!(retrieved.is_some());
    let i = retrieved.unwrap();
    assert_eq!(i.title, "Updated Title");
    assert_eq!(i.status, IssueStatus::InProgress);
    assert_eq!(i.priority, 1);
    assert_eq!(i.assignee, Some("user@example.com".to_string()));
}

// ============================================================================
// Dependency Operations Tests
// ============================================================================

#[tokio::test]
async fn test_add_dependency() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-17", "Project 17", "/path/17");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = create_test_issue("issue-9", "proj-17", "Issue 1", None);
    let issue2 = create_test_issue("issue-10", "proj-17", "Issue 2", None);

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let dep = create_test_dependency("dep-1", "issue-9", "issue-10");
    repo.add_dependency(&dep)
        .await
        .expect("Failed to add dependency");

    let deps = repo
        .list_dependencies("proj-17")
        .await
        .expect("Failed to list dependencies");
    assert!(deps.iter().any(|d| d.id == "dep-1"));
}

#[tokio::test]
async fn test_list_dependencies() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-18", "Project 18", "/path/18");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = create_test_issue("issue-11", "proj-18", "Issue 1", None);
    let issue2 = create_test_issue("issue-12", "proj-18", "Issue 2", None);
    let issue3 = create_test_issue("issue-13", "proj-18", "Issue 3", None);

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");
    repo.create_issue(&issue3)
        .await
        .expect("Failed to create issue 3");

    let dep1 = create_test_dependency("dep-2", "issue-11", "issue-12");
    let dep2 = create_test_dependency("dep-3", "issue-12", "issue-13");

    repo.add_dependency(&dep1)
        .await
        .expect("Failed to add dependency 1");
    repo.add_dependency(&dep2)
        .await
        .expect("Failed to add dependency 2");

    let deps = repo
        .list_dependencies("proj-18")
        .await
        .expect("Failed to list dependencies");
    assert_eq!(deps.len(), 2);
    assert!(deps.iter().any(|d| d.id == "dep-2"));
    assert!(deps.iter().any(|d| d.id == "dep-3"));
}

#[tokio::test]
async fn test_remove_dependency() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-19", "Project 19", "/path/19");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = create_test_issue("issue-14", "proj-19", "Issue 1", None);
    let issue2 = create_test_issue("issue-15", "proj-19", "Issue 2", None);

    repo.create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");
    repo.create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let dep = create_test_dependency("dep-4", "issue-14", "issue-15");
    repo.add_dependency(&dep)
        .await
        .expect("Failed to add dependency");

    repo.remove_dependency("dep-4")
        .await
        .expect("Failed to remove dependency");

    let deps = repo
        .list_dependencies("proj-19")
        .await
        .expect("Failed to list dependencies");
    assert!(!deps.iter().any(|d| d.id == "dep-4"));
}

// ============================================================================
// Decision Operations Tests
// ============================================================================

#[tokio::test]
async fn test_create_decision() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-20", "Project 20", "/path/20");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let decision = create_test_decision("dec-1", "proj-20", "Test Decision", None);
    repo.create_decision(&decision)
        .await
        .expect("Failed to create decision");

    let retrieved = repo
        .get_decision("dec-1")
        .await
        .expect("Failed to get decision");
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().title, "Test Decision");
}

#[tokio::test]
async fn test_list_decisions() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-21", "Project 21", "/path/21");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let dec1 = create_test_decision("dec-2", "proj-21", "Decision 1", None);
    let dec2 = create_test_decision("dec-3", "proj-21", "Decision 2", None);

    repo.create_decision(&dec1)
        .await
        .expect("Failed to create decision 1");
    repo.create_decision(&dec2)
        .await
        .expect("Failed to create decision 2");

    let decisions = repo
        .list_decisions("proj-21")
        .await
        .expect("Failed to list decisions");
    assert!(decisions.len() >= 2);
    assert!(decisions.iter().any(|d| d.id == "dec-2"));
    assert!(decisions.iter().any(|d| d.id == "dec-3"));
}

#[tokio::test]
async fn test_decision_with_issue() {
    let (repo, _temp) = setup_repository().await;
    let project = create_test_project("proj-22", "Project 22", "/path/22");
    repo.create(&project)
        .await
        .expect("Failed to create project");

    let issue = create_test_issue("issue-16", "proj-22", "Test Issue", None);
    repo.create_issue(&issue)
        .await
        .expect("Failed to create issue");

    let decision = create_test_decision(
        "dec-4",
        "proj-22",
        "Decision for Issue",
        Some("issue-16".to_string()),
    );
    repo.create_decision(&decision)
        .await
        .expect("Failed to create decision");

    let retrieved = repo
        .get_decision("dec-4")
        .await
        .expect("Failed to get decision");
    assert!(retrieved.is_some());
    let d = retrieved.unwrap();
    assert_eq!(d.issue_id, Some("issue-16".to_string()));
}
