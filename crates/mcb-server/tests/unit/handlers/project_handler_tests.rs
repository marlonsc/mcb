use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use mcb_server::handlers::project::ProjectHandler;
use rmcp::handler::server::wrapper::Parameters;

fn create_handler() -> ProjectHandler {
    let ctx = crate::shared_context::shared_app_context();
    ProjectHandler::new(ctx.project_repository())
}

#[tokio::test]
async fn rejects_empty_project_id_for_get() {
    let handler = create_handler();
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
}
