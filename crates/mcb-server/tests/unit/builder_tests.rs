//! Tests for McpServerBuilder

use std::sync::Arc;

use mcb_server::builder::{BuilderError, McpServerBuilder};

use crate::test_utils::mock_services::{
    TestAgentSessionService, TestContextService, TestIndexingService, TestIssueEntityRepository,
    TestMemoryService, TestOrgEntityRepository, TestPlanEntityRepository,
    TestProjectDetectorService, TestProjectRepository, TestSearchService, TestValidationService,
    TestVcsEntityRepository, TestVcsProvider,
};

#[test]
fn test_builder_all_services_provided() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .with_project_service(Arc::new(TestProjectDetectorService::new()))
        .with_project_workflow_service(Arc::new(TestProjectRepository::new()))
        .with_vcs_entity_repository(Arc::new(TestVcsEntityRepository::new()))
        .with_plan_entity_repository(Arc::new(TestPlanEntityRepository::new()))
        .with_issue_entity_repository(Arc::new(TestIssueEntityRepository::new()))
        .with_org_entity_repository(Arc::new(TestOrgEntityRepository::new()))
        .build();

    assert!(result.is_ok());
}

#[test]
fn test_builder_missing_indexing_service() {
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "indexing service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_context_service() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "context service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_search_service() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "search service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_validation_service() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "validation service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_empty() {
    let result = McpServerBuilder::new().build();

    assert!(result.is_err());
}

#[test]
fn test_try_build_success() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let server = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .with_project_service(Arc::new(TestProjectDetectorService::new()))
        .with_project_workflow_service(Arc::new(TestProjectRepository::new()))
        .with_vcs_entity_repository(Arc::new(TestVcsEntityRepository::new()))
        .with_plan_entity_repository(Arc::new(TestPlanEntityRepository::new()))
        .with_issue_entity_repository(Arc::new(TestIssueEntityRepository::new()))
        .with_org_entity_repository(Arc::new(TestOrgEntityRepository::new()))
        .build();

    assert!(server.is_ok());
}

#[test]
fn test_builder_missing_vcs_provider() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_agent_session_service(agent_session_service)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "vcs provider");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_memory_service() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let agent_session_service = Arc::new(TestAgentSessionService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_agent_session_service(agent_session_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "memory service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_missing_agent_session_service() {
    let indexing_service = Arc::new(TestIndexingService::new());
    let context_service = Arc::new(TestContextService::new());
    let search_service = Arc::new(TestSearchService::new());
    let validation_service = Arc::new(TestValidationService::new());
    let memory_service = Arc::new(TestMemoryService::new());
    let vcs_provider = Arc::new(TestVcsProvider::new());

    let result = McpServerBuilder::new()
        .with_indexing_service(indexing_service)
        .with_context_service(context_service)
        .with_search_service(search_service)
        .with_validation_service(validation_service)
        .with_memory_service(memory_service)
        .with_vcs_provider(vcs_provider)
        .build();

    assert!(result.is_err());
    match result {
        Err(BuilderError::MissingDependency(dep)) => {
            assert_eq!(dep, "agent session service");
        }
        _ => panic!("Expected MissingDependency error"),
    }
}

#[test]
fn test_builder_default() {
    let builder = McpServerBuilder::default();
    let result = builder.build();

    assert!(result.is_err());
}
