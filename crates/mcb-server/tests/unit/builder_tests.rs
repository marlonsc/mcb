use mcb_server::builder::{BuilderError, McpServerBuilder};

async fn create_real_services()
-> mcb_infrastructure::di::modules::domain_services::DomainServicesContainer {
    let ctx = crate::shared_context::shared_app_context();
    ctx.build_domain_services()
        .await
        .expect("build domain services")
}

#[tokio::test]
async fn test_builder_all_services_provided() {
    let services = create_real_services().await;

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
    let services = create_real_services().await;

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
    let services = create_real_services().await;

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
