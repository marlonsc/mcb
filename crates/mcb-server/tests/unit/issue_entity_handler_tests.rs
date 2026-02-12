use mcb_infrastructure::config::AppConfig;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_server::args::{IssueEntityAction, IssueEntityArgs, IssueEntityResource};
use mcb_server::handlers::issue_entity::IssueEntityHandler;
use rmcp::handler::server::wrapper::Parameters;

async fn create_handler() -> (IssueEntityHandler, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = AppConfig::default();
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    (
        IssueEntityHandler::new(ctx.issue_entity_repository()),
        temp_dir,
    )
}

#[tokio::test]
async fn list_issues_requires_project_id() {
    let (handler, _temp_dir) = create_handler().await;
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
