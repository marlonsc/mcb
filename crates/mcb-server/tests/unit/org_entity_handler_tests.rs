use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_server::args::{OrgEntityAction, OrgEntityArgs, OrgEntityResource};
use mcb_server::handlers::org_entity::OrgEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

async fn create_handler() -> (OrgEntityHandler, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (OrgEntityHandler::new(ctx.org_entity_repository()), temp_dir)
}

#[tokio::test]
async fn get_user_requires_id_or_email() {
    let (handler, _temp_dir) = create_handler().await;
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
