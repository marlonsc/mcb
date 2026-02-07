//! Tests for ProjectService

use mcb_domain::ports::services::project::ProjectDetectorService;
use mcb_infrastructure::project::service::ProjectService;
use std::path::Path;

#[tokio::test]
async fn test_project_service_creation() {
    let service = ProjectService::new();
    assert_eq!(service, ProjectService::default());
}

#[tokio::test]
async fn test_project_service_detect_all() {
    let service = ProjectService::new();
    let temp_dir = std::env::temp_dir();

    // Should not panic on valid path
    let _result = service.detect_all(temp_dir.as_path()).await;
}
