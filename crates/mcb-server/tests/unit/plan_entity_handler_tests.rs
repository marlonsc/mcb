use mcb_server::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use mcb_server::handlers::entities::PlanEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

fn create_handler() -> PlanEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    PlanEntityHandler::new(ctx.plan_entity_repository())
}

#[tokio::test]
async fn list_plan_versions_requires_plan_id() {
    let handler = create_handler();
    let args = PlanEntityArgs {
        action: PlanEntityAction::List,
        resource: PlanEntityResource::Version,
        id: None,
        org_id: None,
        project_id: None,
        plan_id: None,
        plan_version_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing plan_id");
    assert!(err.message.contains("plan_id required"));
}
