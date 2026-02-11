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

#[allow(unused_imports)]
pub use agent::MockAgentRepository;
pub use agent::MockAgentSessionService;
pub use context::MockContextService;
pub use indexing::MockIndexingService;
#[allow(unused_imports)]
pub use issue_entity::MockIssueEntityService;
#[allow(unused_imports)]
pub use memory::MockMemoryRepository;
pub use memory::MockMemoryService;
#[allow(unused_imports)]
pub use org_entity::MockOrgEntityService;
#[allow(unused_imports)]
pub use plan_entity::MockPlanEntityService;
#[allow(unused_imports)]
pub use project::{MockProjectDetectorService, MockProjectRepository};
pub use search::MockSearchService;
pub use validation::MockValidationService;
pub use vcs::MockVcsProvider;
#[allow(unused_imports)]
pub use vcs_entity::MockVcsEntityService;

#[cfg(test)]
mod constructibility {
    use std::sync::atomic::Ordering;

    use mcb_domain::ports::services::IndexingResult;
    use mcb_domain::ports::services::IndexingStatus;
    use mcb_domain::ports::services::ValidationReport;

    use super::*;

    #[test]
    fn test_all_mocks_constructible() {
        let search = MockSearchService::new()
            .with_results(vec![])
            .with_failure("ok");
        assert!(search.results.lock().unwrap().is_empty());

        let indexing = MockIndexingService::new()
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

        let context = MockContextService::new()
            .with_search_results(vec![])
            .with_dimensions(128)
            .with_failure("ok");
        assert_eq!(context.dimensions, 128);

        let validation = MockValidationService::new()
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

        let validation_alt = MockValidationService::with_violations(vec![]);
        assert!(validation_alt.report.lock().unwrap().violations.is_empty());

        let memory = MockMemoryService::new();
        assert!(memory.observations.lock().unwrap().is_empty());

        let vcs = MockVcsProvider::new().with_failure();
        assert!(vcs.should_fail.load(Ordering::SeqCst));

        let _project = MockProjectDetectorService::new();
    }
}
