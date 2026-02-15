use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use mcb_server::handlers::entities::OrgEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

fn create_handler() -> OrgEntityHandler {
    let ctx = crate::shared_context::shared_app_context();
    OrgEntityHandler::new(ctx.org_entity_repository())
}

#[tokio::test]
async fn get_user_requires_id_or_email() {
    let handler = create_handler();
    let args = OrgEntityArgs {
        action: OrgEntityAction::Get,
        resource: OrgEntityResource::User,
        id: None,
        email: None,
        org_id: None,
        team_id: None,
        user_id: None,
        data: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject missing id/email");
    assert!(err.message.contains("id or email required for user get"));
}
