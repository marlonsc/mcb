use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use mcb_server::handlers::project::ProjectHandler;
use rmcp::handler::server::wrapper::Parameters;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn create_handler() -> TestResult<ProjectHandler> {
    let state = crate::utils::shared_context::shared_mcb_state()?;
    Ok(ProjectHandler::new(
        state.mcp_server.project_workflow_repository(),
    ))
}

#[tokio::test]
async fn rejects_empty_project_id_for_get() -> TestResult {
    let handler = create_handler()?;
    let args = ProjectArgs {
        action: ProjectAction::Get,
        resource: ProjectResource::Project,
        project_id: "  ".to_owned(),
        data: None,
        filters: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject empty project_id");
    assert!(err.message.contains("project_id is required"));
    Ok(())
}
