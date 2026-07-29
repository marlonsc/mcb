//! Tests for `ProjectService`

use std::sync::Arc;

use mcb_domain::ports::ProjectDetectorService;
use mcb_infrastructure::project::{DetectAllFn, ProjectService};
use rstest::rstest;

/// Creates a no-op detection function for testing.
fn noop_detect_fn() -> DetectAllFn {
    Arc::new(|_path: &std::path::Path| Box::pin(async { vec![] }))
}

#[rstest]
#[tokio::test]
async fn test_project_service_creation() {
    let service = ProjectService::new(noop_detect_fn());
    // Verify Debug impl works
    let debug_str = format!("{service:?}");
    assert!(debug_str.contains("ProjectService"));
}

#[rstest]
#[tokio::test]
async fn test_project_service_detect_all() {
    let service = ProjectService::new(noop_detect_fn());
    let temp_dir = std::env::temp_dir();

    // Should not panic on valid path
    let result = service.detect_all(temp_dir.as_path()).await;
    assert!(result.is_empty());
}
