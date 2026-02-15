//! Unit tests for `ValidationService`

use std::path::PathBuf;

use std::fs;

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

    assert!(validators.contains(&"clean_architecture".to_owned()));
    assert!(validators.contains(&"solid".to_owned()));
    assert!(validators.contains(&"quality".to_owned()));
}

#[tokio::test]
async fn test_validate_mcb_workspace_quality_only() {
    let workspace_root = get_workspace_root();
    let service = InfraValidationService::new();

    let result = std::thread::Builder::new()
        .name("validate-quality".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(service.validate(&workspace_root, Some(&["quality".to_owned()]), None))
        })
        .expect("spawn thread")
        .join()
        .expect("thread join");

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_validate_with_specific_validator() {
    let workspace_root = get_workspace_root();
    let service = InfraValidationService::new();

    let result = std::thread::Builder::new()
        .name("validate-specific".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(service.validate(
                    &workspace_root,
                    Some(&["quality".to_owned()]),
                    Some("warning"),
                ))
        })
        .expect("spawn thread")
        .join()
        .expect("thread join");

    assert!(result.is_ok());
    let report = result.unwrap();
    assert!(report.passed || !report.violations.is_empty());
}

#[tokio::test]
async fn test_validate_detects_inline_tests_in_src_via_registry_path() {
    let tmp = tempfile::tempdir().expect("create tempdir");
    let workspace = tmp.path();

    fs::create_dir_all(workspace.join("crates/foo/src")).expect("create crate src");
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/foo\"]\n",
    )
    .expect("write workspace cargo");
    fs::write(
        workspace.join("crates/foo/Cargo.toml"),
        "[package]\nname = \"foo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .expect("write crate cargo");
    fs::write(
        workspace.join("crates/foo/src/lib.rs"),
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn smoke() {\n        assert_eq!(1, 1);\n    }\n}\n",
    )
    .expect("write src lib");

    let service = InfraValidationService::new();
    let report = service
        .validate(workspace, Some(&["hygiene".to_owned()]), Some("warning"))
        .await
        .expect("validate should succeed");

    assert!(report.passed);
    assert!(report.warnings > 0);
    assert!(
        report.violations.iter().any(|v| {
            v.id == "TEST001"
                && v.file.as_deref().is_some_and(|f| {
                    // Use Path components for cross-platform compatibility
                    let suffix = PathBuf::from("crates")
                        .join("foo")
                        .join("src")
                        .join("lib.rs");
                    PathBuf::from(f).ends_with(suffix)
                })
        }),
        "violations={:?}",
        report.violations
    );
}
