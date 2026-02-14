use mcb_infrastructure::config::ConfigLoader;
use mcb_infrastructure::di::bootstrap::init_app;
use mcb_server::builder::{BuilderError, McpServerBuilder};

async fn create_real_services() -> (
    mcb_infrastructure::di::modules::domain_services::DomainServicesContainer,
    tempfile::TempDir,
) {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let mut config = ConfigLoader::new().load().expect("load config");
    config.auth.user_db_path = Some(temp_dir.path().join("test.db"));
    let ctx = init_app(config).await.expect("init app context");
    let services = ctx
        .build_domain_services()
        .await
        .expect("build domain services");
    (services, temp_dir)
}

#[tokio::test]
async fn test_builder_all_services_provided() {
    let (services, _temp_dir) = create_real_services().await;

    let result = McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_vcs_provider(services.vcs_provider)
        .with_project_service(services.project_service)
        .with_project_workflow_service(services.project_repository)
        .with_vcs_entity_repository(services.vcs_entity_repository)
        .with_plan_entity_repository(services.plan_entity_repository)
        .with_issue_entity_repository(services.issue_entity_repository)
        .with_org_entity_repository(services.org_entity_repository)
        .build();

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_builder_missing_indexing_service() {
    let (services, _temp_dir) = create_real_services().await;

    let result = McpServerBuilder::new()
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_vcs_provider(services.vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => assert_eq!(dep, "indexing service"),
        _ => panic!("Expected MissingDependency error"),
    }
}

#[tokio::test]
async fn test_builder_missing_vcs_provider() {
    let (services, _temp_dir) = create_real_services().await;

    let result = McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => assert_eq!(dep, "vcs provider"),
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_empty() {
    let result = McpServerBuilder::new().build();
    assert!(result.is_err());
}

#[test]
fn test_builder_default() {
    let result = McpServerBuilder::default().build();
    assert!(result.is_err());
}
