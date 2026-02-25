use mcb_server::builder::{BuilderError, McpServerBuilder};

async fn create_real_services() -> Result<
    mcb_infrastructure::di::modules::domain_services::DomainServicesContainer,
    Box<dyn std::error::Error>,
> {
    let ctx = crate::utils::shared_context::shared_app_context();
    Ok(ctx.build_domain_services().await?)
}

#[tokio::test]
async fn test_builder_all_services_provided() -> Result<(), Box<dyn std::error::Error>> {
    let services = create_real_services().await?;

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

    result.expect("builder with all services should succeed");
    Ok(())
}

#[tokio::test]
async fn test_builder_missing_indexing_service() -> Result<(), Box<dyn std::error::Error>> {
    let services = create_real_services().await?;

    let result = McpServerBuilder::new()
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .with_vcs_provider(services.vcs_provider)
        .build();

    let err = result.expect_err("builder missing indexing service should fail");
    assert!(
        matches!(
            err,
            BuilderError::MissingDependency(dep) if dep == "indexing service"
        ),
        "expected MissingDependency(indexing service), got: {err:?}"
    );
    Ok(())
}

#[tokio::test]
async fn test_builder_missing_vcs_provider() -> Result<(), Box<dyn std::error::Error>> {
    let services = create_real_services().await?;

    let result = McpServerBuilder::new()
        .with_indexing_service(services.indexing_service)
        .with_context_service(services.context_service)
        .with_search_service(services.search_service)
        .with_validation_service(services.validation_service)
        .with_memory_service(services.memory_service)
        .with_agent_session_service(services.agent_session_service)
        .build();

    let err = result.expect_err("builder missing vcs provider should fail");
    assert!(
        matches!(
            err,
            BuilderError::MissingDependency(dep) if dep == "vcs provider"
        ),
        "expected MissingDependency(vcs provider), got: {err:?}"
    );
    Ok(())
}

#[test]
fn test_builder_empty() {
    let result = McpServerBuilder::new().build();
    let err = result.expect_err("empty builder should fail");
    assert!(
        matches!(err, BuilderError::MissingDependency(_)),
        "expected MissingDependency, got: {err:?}"
    );
}

#[test]
fn test_builder_default() {
    let result = McpServerBuilder::default().build();
    let err = result.expect_err("default builder should fail");
    assert!(
        matches!(err, BuilderError::MissingDependency(_)),
        "expected MissingDependency, got: {err:?}"
    );
}
