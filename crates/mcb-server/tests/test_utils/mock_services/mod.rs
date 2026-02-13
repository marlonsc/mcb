//! Mock service implementations for testing

pub mod agent;
pub mod context;
pub mod indexing;
pub mod issue_entity;
pub mod memory;
pub mod org_entity;
pub mod plan_entity;
pub mod project;
pub mod search;
pub mod validation;
pub mod vcs;
pub mod vcs_entity;

pub use agent::TestAgentRepository;
pub use agent::TestAgentSessionService;
pub use context::TestContextService;
pub use indexing::TestIndexingService;
pub use issue_entity::TestIssueEntityRepository;
pub use memory::TestMemoryRepository;
pub use memory::TestMemoryService;
pub use org_entity::TestOrgEntityRepository;
pub use plan_entity::TestPlanEntityRepository;
pub use project::{TestProjectDetectorService, TestProjectRepository};
pub use search::TestSearchService;
pub use validation::TestValidationService;
pub use vcs::TestVcsProvider;
pub use vcs_entity::TestVcsEntityRepository;

#[cfg(test)]
mod constructibility {
    use std::sync::atomic::Ordering;

    use mcb_domain::ports::services::IndexingResult;
    use mcb_domain::ports::services::IndexingStatus;
    use mcb_domain::ports::services::ValidationReport;

    use super::*;

    #[test]
    fn test_all_mocks_constructible() {
        let search = TestSearchService::new()
            .with_results(vec![])
            .with_failure("ok");
        assert!(search.results.lock().unwrap().is_empty());

        let indexing = TestIndexingService::new()
            .with_result(IndexingResult {
                files_processed: 0,
                chunks_created: 0,
                files_skipped: 0,
                errors: vec![],
                operation_id: None,
                status: "ok".to_string(),
            })
            .with_status(IndexingStatus {
                is_indexing: false,
                progress: 0.0,
                current_file: None,
                total_files: 0,
                processed_files: 0,
            });
        assert!(
            indexing
                .indexing_result
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|r| !r.status.is_empty())
        );

        let context = TestContextService::new()
            .with_search_results(vec![])
            .with_dimensions(128)
            .with_failure("ok");
        assert_eq!(context.dimensions, 128);

        let validation = TestValidationService::new()
            .with_report(ValidationReport {
                total_violations: 0,
                errors: 0,
                warnings: 0,
                infos: 0,
                violations: vec![],
                passed: true,
            })
            .with_validators(vec![])
            .with_failure("ok");
        assert!(validation.report.lock().unwrap().passed);

        let validation_alt = TestValidationService::with_violations(vec![]);
        assert!(validation_alt.report.lock().unwrap().violations.is_empty());

        let memory = TestMemoryService::new();
        assert!(memory.observations.lock().unwrap().is_empty());

        let _agent_repo = TestAgentRepository::new();
        let _memory_repo = TestMemoryRepository::new();

        let vcs = TestVcsProvider::new().with_failure();
        assert!(vcs.should_fail.load(Ordering::SeqCst));

        let _project = TestProjectDetectorService::new();
    }
}
