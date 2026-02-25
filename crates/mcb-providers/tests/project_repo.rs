#![allow(clippy::expect_used, missing_docs)]

use mcb_domain::entities::Project;
use mcb_domain::entities::project::{
    DependencyType, IssueFilter, IssueStatus, IssueType, PhaseStatus, ProjectDecision,
    ProjectDependency, ProjectIssue, ProjectPhase,
};
use mcb_domain::ports::ProjectRepository;
use mcb_providers::database::seaorm::repos::project::SeaOrmProjectRepository;
use mcb_providers::migration::Migrator;
use sea_orm::{ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

const ORG_ID: &str = "org-test";
const PROJECT_ID: &str = "proj-1";
const USER_ID: &str = "user-1";

async fn setup() -> TestResult<(DatabaseConnection, SeaOrmProjectRepository)> {
    let db = Database::connect("sqlite::memory:").await?;
    Migrator::up(&db, None).await?;

    db.execute_unprepared(
        "INSERT INTO organizations (id, name, slug, settings_json, created_at, updated_at) VALUES ('org-test', 'Org Test', 'org-test', '{}', 1, 1)",
    )
    .await?;

    db.execute_unprepared(
        "INSERT INTO users (id, org_id, email, display_name, role, created_at, updated_at) VALUES ('user-1', 'org-test', 'user@example.com', 'User Test', 'member', 1, 1)",
    )
    .await?;

    let repo = SeaOrmProjectRepository::new(db.clone());
    Ok((db, repo))
}

fn sample_project() -> Project {
    Project {
        id: PROJECT_ID.to_owned(),
        org_id: ORG_ID.to_owned(),
        name: "Project One".to_owned(),
        path: "/tmp/project-one".to_owned(),
        created_at: 10,
        updated_at: 10,
    }
}

fn sample_phase() -> ProjectPhase {
    ProjectPhase {
        id: "phase-1".to_owned(),
        project_id: PROJECT_ID.to_owned(),
        name: "MVP".to_owned(),
        description: "Ship first version".to_owned(),
        sequence: 1,
        status: PhaseStatus::Planned,
        started_at: None,
        completed_at: None,
        created_at: 11,
        updated_at: 11,
    }
}

fn sample_issue(id: &str, title: &str, labels: Vec<String>) -> ProjectIssue {
    ProjectIssue {
        id: id.to_owned(),
        org_id: ORG_ID.to_owned(),
        project_id: PROJECT_ID.to_owned(),
        created_by: USER_ID.to_owned(),
        phase_id: Some("phase-1".to_owned()),
        title: title.to_owned(),
        description: format!("desc {title}"),
        issue_type: IssueType::Task,
        status: IssueStatus::Open,
        priority: 2,
        assignee: Some(USER_ID.to_owned()),
        labels,
        estimated_minutes: Some(60),
        actual_minutes: None,
        notes: String::new(),
        design: String::new(),
        parent_issue_id: None,
        closed_at: None,
        closed_reason: String::new(),
        created_at: 20,
        updated_at: 20,
    }
}

fn sample_dependency(id: &str, from: &str, to: &str) -> ProjectDependency {
    ProjectDependency {
        id: id.to_owned(),
        from_issue_id: from.to_owned(),
        to_issue_id: to.to_owned(),
        dependency_type: DependencyType::Blocks,
        created_at: 30,
    }
}

fn sample_decision() -> ProjectDecision {
    ProjectDecision {
        id: "dec-1".to_owned(),
        project_id: PROJECT_ID.to_owned(),
        issue_id: Some("iss-1".to_owned()),
        title: "Database strategy".to_owned(),
        context: "Need deterministic local setup".to_owned(),
        decision: "Use sqlite for dev".to_owned(),
        consequences: "Need migration coverage".to_owned(),
        created_at: 40,
    }
}

#[tokio::test]
async fn project_crud_works() -> TestResult {
    let (_db, repo) = setup().await?;
    let mut project = sample_project();

    repo.create(&project).await?;
    assert_eq!(
        repo.get_by_id(ORG_ID, PROJECT_ID).await?.name,
        "Project One"
    );
    assert_eq!(
        repo.get_by_name(ORG_ID, "Project One").await?.id,
        PROJECT_ID
    );
    assert_eq!(
        repo.get_by_path(ORG_ID, "/tmp/project-one").await?.id,
        PROJECT_ID
    );

    project.name = "Project Renamed".to_owned();
    project.updated_at = 99;
    repo.update(&project).await?;
    assert_eq!(
        repo.get_by_id(ORG_ID, PROJECT_ID).await?.name,
        "Project Renamed"
    );

    assert_eq!(repo.list(ORG_ID).await?.len(), 1);
    repo.delete(ORG_ID, PROJECT_ID).await?;
    assert!(repo.get_by_id(ORG_ID, PROJECT_ID).await.is_err());

    Ok(())
}

#[tokio::test]
async fn phase_crud_works() -> TestResult {
    let (_db, repo) = setup().await?;
    repo.create(&sample_project()).await?;
    let mut phase = sample_phase();

    repo.create_phase(&phase).await?;
    assert_eq!(repo.get_phase_by_id("phase-1").await?.name, "MVP");
    assert_eq!(repo.list_phases(PROJECT_ID).await?.len(), 1);

    phase.status = PhaseStatus::InProgress;
    phase.started_at = Some(100);
    phase.updated_at = 101;
    repo.update_phase(&phase).await?;

    let reloaded = repo.get_phase_by_id("phase-1").await?;
    assert_eq!(reloaded.status, PhaseStatus::InProgress);
    assert_eq!(reloaded.started_at, Some(100));

    repo.delete_phase("phase-1").await?;
    assert!(repo.get_phase_by_id("phase-1").await.is_err());
    Ok(())
}

#[tokio::test]
async fn issue_crud_and_filters_work() -> TestResult {
    let (_db, repo) = setup().await?;
    repo.create(&sample_project()).await?;
    repo.create_phase(&sample_phase()).await?;

    let mut issue1 = sample_issue("iss-1", "Implement auth", vec!["backend".to_owned()]);
    let issue2 = sample_issue(
        "iss-2",
        "Fix login UI",
        vec!["frontend".to_owned(), "bug".to_owned()],
    );

    repo.create_issue(&issue1).await?;
    repo.create_issue(&issue2).await?;
    assert_eq!(
        repo.get_issue_by_id(ORG_ID, "iss-1").await?.title,
        "Implement auth"
    );
    assert_eq!(repo.list_issues(ORG_ID, PROJECT_ID).await?.len(), 2);

    issue1.status = IssueStatus::InProgress;
    issue1.priority = 1;
    issue1.updated_at = 25;
    repo.update_issue(&issue1).await?;

    let filtered = repo
        .list_issues_filtered(
            ORG_ID,
            &IssueFilter {
                project_id: Some(PROJECT_ID.to_owned()),
                status: Some(IssueStatus::InProgress),
                assignee: Some(USER_ID.to_owned()),
                label: Some("backend".to_owned()),
                limit: Some(10),
                ..IssueFilter::default()
            },
        )
        .await?;
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].id, "iss-1");

    repo.delete_issue(ORG_ID, "iss-2").await?;
    assert!(repo.get_issue_by_id(ORG_ID, "iss-2").await.is_err());
    Ok(())
}

#[tokio::test]
async fn dependency_crud_and_traversal_work() -> TestResult {
    let (_db, repo) = setup().await?;
    repo.create(&sample_project()).await?;
    repo.create_phase(&sample_phase()).await?;

    let issue1 = sample_issue("iss-1", "A", vec![]);
    let issue2 = sample_issue("iss-2", "B", vec![]);
    let issue3 = sample_issue("iss-3", "C", vec![]);
    repo.create_issue(&issue1).await?;
    repo.create_issue(&issue2).await?;
    repo.create_issue(&issue3).await?;

    let dep1 = sample_dependency("dep-1", "iss-1", "iss-2");
    let dep2 = sample_dependency("dep-2", "iss-2", "iss-3");
    repo.create_dependency(&dep1).await?;
    repo.create_dependency(&dep2).await?;

    let direct = repo.list_dependencies("iss-2").await?;
    assert_eq!(direct.len(), 2);

    let traversed = repo.traverse_dependencies("iss-1", 5).await?;
    assert_eq!(traversed.len(), 2);
    assert!(traversed.iter().any(|d| d.id == "dep-1"));
    assert!(traversed.iter().any(|d| d.id == "dep-2"));

    repo.delete_dependency("dep-2").await?;
    let direct_after_delete = repo.list_dependencies("iss-2").await?;
    assert_eq!(direct_after_delete.len(), 1);
    Ok(())
}

#[tokio::test]
async fn decision_crud_works() -> TestResult {
    let (_db, repo) = setup().await?;
    repo.create(&sample_project()).await?;
    repo.create_phase(&sample_phase()).await?;
    repo.create_issue(&sample_issue("iss-1", "Auth", vec![]))
        .await?;

    let mut decision = sample_decision();
    repo.create_decision(&decision).await?;

    assert_eq!(
        repo.get_decision_by_id("dec-1").await?.title,
        "Database strategy"
    );
    assert_eq!(repo.list_decisions(PROJECT_ID).await?.len(), 1);

    decision.title = "Storage strategy".to_owned();
    decision.decision = "Use sqlite + snapshots".to_owned();
    repo.update_decision(&decision).await?;

    let updated = repo.get_decision_by_id("dec-1").await?;
    assert_eq!(updated.title, "Storage strategy");

    repo.delete_decision("dec-1").await?;
    assert!(repo.get_decision_by_id("dec-1").await.is_err());
    Ok(())
}
