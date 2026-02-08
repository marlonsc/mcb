//! Unit tests for ValidationService

use std::path::PathBuf;

use mcb_domain::ports::services::ValidationServiceInterface;
use mcb_infrastructure::validation::InfraValidationService;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[tokio::test]
async fn test_list_validators() {
    let service = InfraValidationService::new();
    let validators = service.list_validators().await.unwrap();

    assert!(validators.contains(&"clean_architecture".to_string()));
    assert!(validators.contains(&"solid".to_string()));
    assert!(validators.contains(&"quality".to_string()));
}

#[tokio::test]
async fn test_validate_mcb_workspace() {
    let workspace_root = get_workspace_root();
    let service = InfraValidationService::new();
    let result = service.validate(&workspace_root, None, None).await;

    // Should complete without panic (may have violations, that's OK)
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_with_specific_validator() {
    let workspace_root = get_workspace_root();
    let service = InfraValidationService::new();
    let result = service
        .validate(
            &workspace_root,
            Some(&["quality".to_string()]),
            Some("warning"),
        )
        .await;

    assert!(result.is_ok());
    let report = result.unwrap();
    // Quality validator should return a valid report
    assert!(report.passed || !report.violations.is_empty());
}
