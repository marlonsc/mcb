/// Golden tests: Project operations handler
/// Verifies project handler routing for all resource types and input validation
use crate::utils::test_fixtures::create_test_mcp_server;
use mcb_domain::utils::tests::utils::TestResult;
use mcb_domain::utils::text::extract_text_from;
use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::CallToolResult;
use rstest::rstest;
use serde_json::Value;

fn base_args(action: ProjectAction, resource: ProjectResource) -> ProjectArgs {
    ProjectArgs {
        action,
        resource,
        project_id: Some("test-project".to_owned()),
        id: None,
        issue_id: None,
        data: None,
        filters: None,
        org_id: None,
    }
}

fn result_json(result: &CallToolResult) -> Value {
    serde_json::from_str(&extract_text_from(&result.content)).expect("project response is JSON")
}

#[rstest]
#[tokio::test]
async fn golden_project_create_get_update_delete() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut create = base_args(ProjectAction::Create, ProjectResource::Project);
    create.data = Some(serde_json::json!({
        "name": "Golden Workflow Project",
        "path": "/tmp/golden-workflow-project"
    }));
    let created = server.project_handler().handle(Parameters(create)).await?;
    let created_json = result_json(&created);
    assert_eq!(created_json["id"], "test-project");
    assert_eq!(created_json["name"], "Golden Workflow Project");

    let get = server
        .project_handler()
        .handle(Parameters(base_args(
            ProjectAction::Get,
            ProjectResource::Project,
        )))
        .await?;
    let get_json = result_json(&get);
    assert_eq!(get_json["path"], "/tmp/golden-workflow-project");

    let mut update = base_args(ProjectAction::Update, ProjectResource::Project);
    update.data = Some(serde_json::json!({
        "name": "Golden Workflow Project Updated",
        "path": "/tmp/golden-workflow-project-updated"
    }));
    let updated = server.project_handler().handle(Parameters(update)).await?;
    let updated_json = result_json(&updated);
    assert_eq!(updated_json["name"], "Golden Workflow Project Updated");

    let listed = server
        .project_handler()
        .handle(Parameters(base_args(
            ProjectAction::List,
            ProjectResource::Project,
        )))
        .await?;
    let listed_json = result_json(&listed);
    assert!(
        listed_json
            .as_array()
            .is_some_and(|items| items.iter().any(|item| item["id"] == "test-project")),
        "project list should contain created project: {listed_json}"
    );

    let deleted = server
        .project_handler()
        .handle(Parameters(base_args(
            ProjectAction::Delete,
            ProjectResource::Project,
        )))
        .await?;
    let deleted_json = result_json(&deleted);
    assert_eq!(deleted_json["deleted"], true);
    assert_eq!(deleted_json["id"], "test-project");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_phase_lifecycle() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut project = base_args(ProjectAction::Create, ProjectResource::Project);
    project.data = Some(serde_json::json!({
        "name": "Phase Host Project",
        "path": "/tmp/golden-phase-host"
    }));
    server.project_handler().handle(Parameters(project)).await?;

    let mut args = base_args(ProjectAction::Create, ProjectResource::Phase);
    args.data = Some(serde_json::json!({
        "id": "phase-001",
        "project_id": "test-project",
        "name": "Phase 1 - Foundation",
        "description": "Initial project setup",
        "sequence": 1,
        "status": "Planned",
        "created_at": 0,
        "updated_at": 0
    }));

    let result = server.project_handler().handle(Parameters(args)).await;

    assert!(result.is_ok(), "Create Phase should succeed: {result:?}");
    let created_json = result_json(&result?);
    assert_eq!(created_json["name"], "Phase 1 - Foundation");

    let mut get_args = base_args(ProjectAction::Get, ProjectResource::Phase);
    get_args.id = Some("phase-001".to_owned());
    let get = server
        .project_handler()
        .handle(Parameters(get_args))
        .await?;
    let get_json = result_json(&get);
    assert_eq!(get_json["description"], "Initial project setup");

    let mut update_args = base_args(ProjectAction::Update, ProjectResource::Phase);
    update_args.data = Some(serde_json::json!({
        "id": "phase-001",
        "project_id": "test-project",
        "name": "Phase 1 - Foundation",
        "description": "Foundation completed",
        "sequence": 1,
        "status": "Completed",
        "started_at": 100,
        "completed_at": 200,
        "created_at": 0,
        "updated_at": 1
    }));
    let updated = server
        .project_handler()
        .handle(Parameters(update_args))
        .await?;
    assert_eq!(extract_text_from(&updated.content), "updated");

    let list_args = base_args(ProjectAction::List, ProjectResource::Phase);
    let listed = server
        .project_handler()
        .handle(Parameters(list_args))
        .await?;
    let listed_json = result_json(&listed);
    assert!(
        listed_json.as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["id"] == "phase-001" && item["status"] == "Completed")
        }),
        "phase list should contain updated phase: {listed_json}"
    );

    let mut delete_args = base_args(ProjectAction::Delete, ProjectResource::Phase);
    delete_args.id = Some("phase-001".to_owned());
    let deleted = server
        .project_handler()
        .handle(Parameters(delete_args))
        .await?;
    assert_eq!(extract_text_from(&deleted.content), "deleted");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_decision_lifecycle() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut project = base_args(ProjectAction::Create, ProjectResource::Project);
    project.data = Some(serde_json::json!({
        "name": "Decision Host Project",
        "path": "/tmp/golden-decision-host"
    }));
    server.project_handler().handle(Parameters(project)).await?;

    let mut args = base_args(ProjectAction::Create, ProjectResource::Decision);
    args.data = Some(serde_json::json!({
        "id": "dec-001",
        "project_id": "test-project",
        "title": "Use JWT for authentication",
        "context": "Need stateless auth for microservices",
        "decision": "Adopt JWT with refresh tokens",
        "consequences": "Must handle token rotation",
        "created_at": 0
    }));

    let result = server.project_handler().handle(Parameters(args)).await;

    assert!(result.is_ok(), "Create Decision should succeed: {result:?}");
    let created_json = result_json(&result?);
    assert_eq!(created_json["title"], "Use JWT for authentication");

    let mut get_args = base_args(ProjectAction::Get, ProjectResource::Decision);
    get_args.id = Some("dec-001".to_owned());
    let get = server
        .project_handler()
        .handle(Parameters(get_args))
        .await?;
    let get_json = result_json(&get);
    assert_eq!(get_json["decision"], "Adopt JWT with refresh tokens");

    let mut update_args = base_args(ProjectAction::Update, ProjectResource::Decision);
    update_args.data = Some(serde_json::json!({
        "id": "dec-001",
        "project_id": "test-project",
        "issue_id": null,
        "title": "Use signed JWT for authentication",
        "context": "Need stateless auth for microservices",
        "decision": "Adopt signed JWT with refresh tokens",
        "consequences": "Must handle token rotation and key rollover",
        "created_at": 0
    }));
    let updated = server
        .project_handler()
        .handle(Parameters(update_args))
        .await?;
    assert_eq!(extract_text_from(&updated.content), "updated");

    let list_args = base_args(ProjectAction::List, ProjectResource::Decision);
    let listed = server
        .project_handler()
        .handle(Parameters(list_args))
        .await?;
    let listed_json = result_json(&listed);
    assert!(
        listed_json.as_array().is_some_and(|items| {
            items.iter().any(|item| {
                item["id"] == "dec-001" && item["title"] == "Use signed JWT for authentication"
            })
        }),
        "decision list should contain updated decision: {listed_json}"
    );

    let mut delete_args = base_args(ProjectAction::Delete, ProjectResource::Decision);
    delete_args.id = Some("dec-001".to_owned());
    let deleted = server
        .project_handler()
        .handle(Parameters(delete_args))
        .await?;
    assert_eq!(extract_text_from(&deleted.content), "deleted");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_dependency_lifecycle() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut project = base_args(ProjectAction::Create, ProjectResource::Project);
    project.data = Some(serde_json::json!({
        "name": "Dependency Host Project",
        "path": "/tmp/golden-dependency-host"
    }));
    server.project_handler().handle(Parameters(project)).await?;

    for (id, title) in [
        ("issue-source", "Source dependency issue"),
        ("issue-target", "Target dependency issue"),
    ] {
        let mut issue = base_args(ProjectAction::Create, ProjectResource::Issue);
        issue.data = Some(serde_json::json!({
            "id": id,
            "org_id": "",
            "project_id": "test-project",
            "created_by": "golden-test",
            "phase_id": null,
            "title": title,
            "description": title,
            "issue_type": "Task",
            "status": "Open",
            "priority": 2,
            "assignee": null,
            "labels": [],
            "estimated_minutes": null,
            "actual_minutes": null,
            "notes": "",
            "design": "",
            "parent_issue_id": null,
            "closed_at": null,
            "closed_reason": "",
            "created_at": 0,
            "updated_at": 0
        }));
        server.project_handler().handle(Parameters(issue)).await?;
    }

    let mut create_dep = base_args(ProjectAction::Create, ProjectResource::Dependency);
    create_dep.data = Some(serde_json::json!({
        "id": "dep-001",
        "from_issue_id": "issue-source",
        "to_issue_id": "issue-target",
        "dependency_type": "Blocks",
        "created_at": 0
    }));
    let created = server
        .project_handler()
        .handle(Parameters(create_dep))
        .await?;
    let created_json = result_json(&created);
    assert_eq!(created_json["id"], "dep-001");

    let mut list_dep = base_args(ProjectAction::List, ProjectResource::Dependency);
    list_dep.issue_id = Some("issue-source".to_owned());
    let listed = server
        .project_handler()
        .handle(Parameters(list_dep))
        .await?;
    let listed_json = result_json(&listed);
    assert!(
        listed_json.as_array().is_some_and(|items| {
            items
                .iter()
                .any(|item| item["id"] == "dep-001" && item["to_issue_id"] == "issue-target")
        }),
        "dependency list should contain created dependency: {listed_json}"
    );

    let mut delete_dep = base_args(ProjectAction::Delete, ProjectResource::Dependency);
    delete_dep.id = Some("dep-001".to_owned());
    let deleted = server
        .project_handler()
        .handle(Parameters(delete_dep))
        .await?;
    assert_eq!(extract_text_from(&deleted.content), "deleted");
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_missing_project_id() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut args = base_args(ProjectAction::Get, ProjectResource::Project);
    args.project_id = Some(String::new());

    let result = server.project_handler().handle(Parameters(args)).await;

    assert!(
        result.is_err(),
        "Get without project_id should fail: {result:?}"
    );
    let err = result.expect_err("missing project_id should return error");
    assert!(
        err.message.contains("project_id is required"),
        "error should mention project_id is required, got: {}",
        err.message
    );
    Ok(())
}
