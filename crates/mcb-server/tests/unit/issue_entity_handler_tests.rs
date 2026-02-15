use mcb_server::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use mcb_server::handlers::entities::IssueEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

fn create_handler() -> IssueEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    IssueEntityHandler::new(ctx.issue_entity_repository())
}

#[tokio::test]
async fn list_issues_requires_project_id() {
    let handler = create_handler();
    let args = IssueEntityArgs {
        action: IssueEntityAction::List,
        resource: IssueEntityResource::Issue,
        id: None,
        issue_id: None,
        label_id: None,
        org_id: None,
        project_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing project_id");
    assert!(err.message.contains("project_id required for list"));
}
