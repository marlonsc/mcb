use mcb_server::builder::{BuilderError, McpServerBuilder};

use crate::utils::domain_services::create_real_domain_services;

async fn create_real_services() -> Option<(mcb_server::state::McbState, tempfile::TempDir)> {
    create_real_domain_services().await
}

#[tokio::test]
async fn test_builder_all_services_provided() -> Result<(), Box<dyn std::error::Error>> {
    let Some((state, _temp)) = create_real_services().await else {
        return Ok(());
    };
    let s = &state.mcp_server;

    let result = McpServerBuilder::new()
        .with_indexing_service(s.indexing_service())
        .with_context_service(s.context_service())
        .with_search_service(s.search_service())
        .with_validation_service(s.validation_service())
        .with_memory_service(s.memory_service())
        .with_agent_session_service(s.agent_session_service())
        .with_vcs_provider(s.vcs_provider())
        .with_project_service(s.project_service())
        .with_project_workflow_service(s.project_workflow_repository())
        .with_vcs_entity_repository(s.vcs_entity_repository())
        .with_plan_entity_repository(s.plan_entity_repository())
        .with_issue_entity_repository(s.issue_entity_repository())
        .with_org_entity_repository(s.org_entity_repository())
        .build();

    result.expect("builder with all services should succeed");
    Ok(())
}

#[tokio::test]
async fn test_builder_missing_indexing_service() -> Result<(), Box<dyn std::error::Error>> {
    let Some((state, _temp)) = create_real_services().await else {
        return Ok(());
    };
    let s = &state.mcp_server;

    let result = McpServerBuilder::new()
        .with_context_service(s.context_service())
        .with_search_service(s.search_service())
        .with_validation_service(s.validation_service())
        .with_memory_service(s.memory_service())
        .with_agent_session_service(s.agent_session_service())
        .with_vcs_provider(s.vcs_provider())
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
    let Some((state, _temp)) = create_real_services().await else {
        return Ok(());
    };
    let s = &state.mcp_server;

    let result = McpServerBuilder::new()
        .with_indexing_service(s.indexing_service())
        .with_context_service(s.context_service())
        .with_search_service(s.search_service())
        .with_validation_service(s.validation_service())
        .with_memory_service(s.memory_service())
        .with_agent_session_service(s.agent_session_service())
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
