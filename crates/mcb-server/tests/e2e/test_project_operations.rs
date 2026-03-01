/// Golden tests: Project operations handler
/// Verifies project handler routing, unsupported action errors, and input validation
use crate::utils::test_fixtures::create_test_mcp_server;
use mcb_domain::test_utils::TestResult;
use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use rmcp::handler::server::wrapper::Parameters;
use rstest::rstest;

fn base_args(action: ProjectAction, resource: ProjectResource) -> ProjectArgs {
    ProjectArgs {
        action,
        resource,
        project_id: Some("test-project".to_owned()),
        data: None,
        filters: None,
    }
}

#[rstest]
#[tokio::test]
async fn golden_project_create_phase() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut args = base_args(ProjectAction::Create, ProjectResource::Phase);
    args.data = Some(serde_json::json!({
        "name": "Phase 1 - Foundation",
        "description": "Initial project setup",
        "sequence": 1,
        "status": "planned"
    }));

    let result = server.project_handler().handle(Parameters(args)).await;

    // Create+Phase is not yet implemented — handler returns unsupported error
    assert!(
        result.is_err(),
        "Create Phase should return error (unsupported): {result:?}"
    );
    let err = result.expect_err("Create Phase should be unsupported");
    assert!(
        err.message.contains("Unsupported action"),
        "error should mention unsupported action, got: {}",
        err.message
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_list_phases() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let args = base_args(ProjectAction::List, ProjectResource::Phase);

    let result = server.project_handler().handle(Parameters(args)).await;

    // List+Phase is not yet implemented — handler returns unsupported error
    assert!(
        result.is_err(),
        "List Phases should return error (unsupported): {result:?}"
    );
    let err = result.expect_err("List Phases should be unsupported");
    assert!(
        err.message.contains("Unsupported action"),
        "error should mention unsupported action, got: {}",
        err.message
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn golden_project_create_decision() -> TestResult {
    let (server, _td) = create_test_mcp_server().await?;

    let mut args = base_args(ProjectAction::Create, ProjectResource::Decision);
    args.data = Some(serde_json::json!({
        "title": "Use JWT for authentication",
        "context": "Need stateless auth for microservices",
        "decision": "Adopt JWT with refresh tokens",
        "consequences": "Must handle token rotation"
    }));

    let result = server.project_handler().handle(Parameters(args)).await;

    // Create+Decision is not yet implemented — handler returns unsupported error
    assert!(
        result.is_err(),
        "Create Decision should return error (unsupported): {result:?}"
    );
    let err = result.expect_err("Create Decision should be unsupported");
    assert!(
        err.message.contains("Unsupported action"),
        "error should mention unsupported action, got: {}",
        err.message
    );
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
