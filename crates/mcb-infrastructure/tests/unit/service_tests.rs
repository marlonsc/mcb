//! Tests for ProjectService

use mcb_domain::ports::services::project::ProjectDetectorService;
use mcb_infrastructure::project::ProjectService;

#[tokio::test]
async fn test_project_service_creation() {
    let service = ProjectService::new();
    assert_eq!(service, ProjectService);
}

#[tokio::test]
async fn test_project_service_detect_all() {
    let service = ProjectService::new();
    let temp_dir = std::env::temp_dir();

    // Should not panic on valid path
    let result = service.detect_all(temp_dir.as_path()).await;
    assert!(result.is_ok());
}
