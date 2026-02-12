use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_server::args::{ProjectAction, ProjectArgs, ProjectResource};
use mcb_server::handlers::project::ProjectHandler;
use rmcp::handler::server::wrapper::Parameters;

async fn create_handler() -> (ProjectHandler, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (ProjectHandler::new(ctx.project_repository()), temp_dir)
}

#[tokio::test]
async fn rejects_empty_project_id_for_get() {
    let (handler, _temp_dir) = create_handler().await;
    let args = ProjectArgs {
        action: ProjectAction::Get,
        resource: ProjectResource::Project,
        project_id: "  ".to_string(),
        data: None,
        filters: None,
    };

    let err = handler
        .handle(Parameters(args))
        .await
        .expect_err("must reject empty project_id");
    assert!(err.message.contains("project_id is required"));
}
