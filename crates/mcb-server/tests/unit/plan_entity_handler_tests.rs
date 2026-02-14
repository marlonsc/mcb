use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_server::args::{PlanEntityAction, PlanEntityArgs, PlanEntityResource};
use mcb_server::handlers::entities::PlanEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

async fn create_handler() -> (PlanEntityHandler, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (
        PlanEntityHandler::new(ctx.plan_entity_repository()),
        temp_dir,
    )
}

#[tokio::test]
async fn list_plan_versions_requires_plan_id() {
    let (handler, _temp_dir) = create_handler().await;
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
