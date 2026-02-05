use mcb_domain::entities::project::{
    DependencyType, IssueStatus, IssueType, PhaseStatus, Project, ProjectDecision, ProjectIssue,
    ProjectPhase,
};
use mcb_domain::ports::repositories::ProjectRepository;
use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use mcb_server::handlers::ProjectHandler;
use rmcp::handler::server::wrapper::Parameters;
use serde_json::json;
use std::sync::Arc;

use crate::test_utils::mock_services::MockProjectRepository;

#[tokio::test]
async fn test_project_create_action() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let args = ProjectArgs {
        action: ProjectAction::Create,
        resource: ProjectResource::Phase,
        project_id: "test-project".to_string(),
        resource_id: None,
        phase_id: None,
        status: None,
        priority: None,
        limit: None,
        data: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));

    let project = repository
        .get_by_id("test-project")
        .await
        .expect("Failed to get project")
        .expect("Project should exist");
    assert_eq!(project.id, "test-project");
    assert_eq!(project.name, "test-project");
}

#[tokio::test]
async fn test_project_list_phases() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let phase = ProjectPhase {
        id: "phase-1".to_string(),
        project_id: "test-project".to_string(),
        name: "Phase 1".to_string(),
        description: "Test phase".to_string(),
        sequence: 1,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create_phase(&phase)
        .await
        .expect("Failed to create phase");

    let args = ProjectArgs {
        action: ProjectAction::List,
        resource: ProjectResource::Phase,
        project_id: "test-project".to_string(),
        resource_id: None,
        phase_id: None,
        status: None,
        priority: None,
        limit: Some(10),
        data: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_project_create_phase() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let phase = ProjectPhase {
        id: "phase-1".to_string(),
        project_id: "test-project".to_string(),
        name: "Old Phase".to_string(),
        description: "Test phase".to_string(),
        sequence: 1,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create_phase(&phase)
        .await
        .expect("Failed to create phase");

    let args = ProjectArgs {
        action: ProjectAction::Update,
        resource: ProjectResource::Phase,
        project_id: "test-project".to_string(),
        resource_id: Some("phase-1".to_string()),
        phase_id: None,
        status: None,
        priority: None,
        limit: None,
        data: Some(json!({
            "name": "New Phase"
        })),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));

    let updated_phase = repository
        .get_phase("phase-1")
        .await
        .expect("Failed to get phase")
        .expect("Phase should exist");
    assert_eq!(updated_phase.name, "New Phase");
}

#[tokio::test]
async fn test_project_create_issue() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let phase = ProjectPhase {
        id: "phase-1".to_string(),
        project_id: "test-project".to_string(),
        name: "Phase 1".to_string(),
        description: "Test phase".to_string(),
        sequence: 1,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create_phase(&phase)
        .await
        .expect("Failed to create phase");

    let issue = ProjectIssue {
        id: "issue-1".to_string(),
        project_id: "test-project".to_string(),
        phase_id: Some("phase-1".to_string()),
        title: "Test Issue".to_string(),
        description: "Test description".to_string(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue)
        .await
        .expect("Failed to create issue");

    let args = ProjectArgs {
        action: ProjectAction::List,
        resource: ProjectResource::Issue,
        project_id: "test-project".to_string(),
        resource_id: None,
        phase_id: Some("phase-1".to_string()),
        status: None,
        priority: None,
        limit: Some(10),
        data: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_project_filter_issues() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = ProjectIssue {
        id: "issue-1".to_string(),
        project_id: "test-project".to_string(),
        phase_id: None,
        title: "High Priority Issue".to_string(),
        description: "Bug report".to_string(),
        issue_type: IssueType::Bug,
        status: IssueStatus::Open,
        priority: 0,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");

    let issue2 = ProjectIssue {
        id: "issue-2".to_string(),
        project_id: "test-project".to_string(),
        phase_id: None,
        title: "Low Priority Issue".to_string(),
        description: "Task".to_string(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 3,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let args = ProjectArgs {
        action: ProjectAction::List,
        resource: ProjectResource::Issue,
        project_id: "test-project".to_string(),
        resource_id: None,
        phase_id: None,
        status: Some("open".to_string()),
        priority: Some(0),
        limit: Some(10),
        data: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_project_add_dependency() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let issue1 = ProjectIssue {
        id: "issue-1".to_string(),
        project_id: "test-project".to_string(),
        phase_id: None,
        title: "Issue 1".to_string(),
        description: "First issue".to_string(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue1)
        .await
        .expect("Failed to create issue 1");

    let issue2 = ProjectIssue {
        id: "issue-2".to_string(),
        project_id: "test-project".to_string(),
        phase_id: None,
        title: "Issue 2".to_string(),
        description: "Second issue".to_string(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue2)
        .await
        .expect("Failed to create issue 2");

    let args = ProjectArgs {
        action: ProjectAction::AddDependency,
        resource: ProjectResource::Dependency,
        project_id: "test-project".to_string(),
        resource_id: Some("issue-1".to_string()),
        phase_id: None,
        status: None,
        priority: None,
        limit: None,
        data: Some(json!({
            "to_id": "issue-2",
            "type": "blocks"
        })),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));

    let dependencies = repository
        .list_dependencies("test-project")
        .await
        .expect("Failed to list dependencies");
    assert_eq!(dependencies.len(), 1);
    assert_eq!(dependencies[0].from_issue_id, "issue-1");
    assert_eq!(dependencies[0].to_issue_id, "issue-2");
    assert_eq!(dependencies[0].dependency_type, DependencyType::Blocks);
}

#[tokio::test]
async fn test_project_list_decisions() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let decision = ProjectDecision {
        id: "decision-1".to_string(),
        project_id: "test-project".to_string(),
        issue_id: None,
        title: "Use Rust for backend".to_string(),
        context: "Performance requirements".to_string(),
        decision: "Chose Rust".to_string(),
        consequences: "Learning curve".to_string(),
        created_at: 1000,
    };
    repository
        .create_decision(&decision)
        .await
        .expect("Failed to create decision");

    let args = ProjectArgs {
        action: ProjectAction::List,
        resource: ProjectResource::Decision,
        project_id: "test-project".to_string(),
        resource_id: None,
        phase_id: None,
        status: None,
        priority: None,
        limit: Some(10),
        data: None,
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));
}

#[tokio::test]
async fn test_project_update_issue_status() {
    let repository = Arc::new(MockProjectRepository::new());
    let handler = ProjectHandler::new(repository.clone());

    let project = Project {
        id: "test-project".to_string(),
        name: "Test Project".to_string(),
        path: "/test/path".to_string(),
        created_at: 1000,
        updated_at: 1000,
    };
    repository
        .create(&project)
        .await
        .expect("Failed to create project");

    let issue = ProjectIssue {
        id: "issue-1".to_string(),
        project_id: "test-project".to_string(),
        phase_id: None,
        title: "Test Issue".to_string(),
        description: "Test description".to_string(),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: None,
        labels: vec![],
        created_at: 1000,
        updated_at: 1000,
        closed_at: None,
    };
    repository
        .create_issue(&issue)
        .await
        .expect("Failed to create issue");

    let args = ProjectArgs {
        action: ProjectAction::Update,
        resource: ProjectResource::Issue,
        project_id: "test-project".to_string(),
        resource_id: Some("issue-1".to_string()),
        phase_id: None,
        status: None,
        priority: None,
        limit: None,
        data: Some(json!({
            "status": "closed"
        })),
    };

    let result = handler.handle(Parameters(args)).await;

    assert!(result.is_ok());
    let response = result.expect("Expected successful response");
    assert!(!response.is_error.unwrap_or(false));

    let updated_issue = repository
        .get_issue("issue-1")
        .await
        .expect("Failed to get issue")
        .expect("Issue should exist");
    assert_eq!(updated_issue.status, IssueStatus::Closed);
}
