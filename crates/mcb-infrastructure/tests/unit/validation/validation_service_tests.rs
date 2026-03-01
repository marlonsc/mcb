//! Unit tests for `ValidationService`

use std::fs;
use std::path::PathBuf;

use mcb_domain::ports::ValidationServiceInterface;
use mcb_domain::registry::services::resolve_validation_service;
use mcb_domain::test_utils::workspace_root;
use rstest::{fixture, rstest};

#[fixture]
fn validation_service()
-> Result<std::sync::Arc<dyn ValidationServiceInterface>, Box<dyn std::error::Error>> {
    Ok(resolve_validation_service(&())?)
}

#[rstest]
#[tokio::test]
async fn test_list_validators() -> Result<(), Box<dyn std::error::Error>> {
    let service = validation_service()?;
    let validators = service.list_validators().await?;

    assert!(validators.contains(&"clean_architecture".to_owned()));
    assert!(validators.contains(&"solid".to_owned()));
    assert!(validators.contains(&"quality".to_owned()));
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_validate_mcb_workspace_quality_only() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let service = validation_service()?;

    let result = std::thread::Builder::new()
        .name("validate-quality".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
                .and_then(|rt| {
                    rt.block_on(service.validate(
                        &workspace_root,
                        Some(&["quality".to_owned()]),
                        None,
                    ))
                })
        })?
        .join()
        .map_err(|_| "thread panicked")?;

    let report = result.expect("quality validation should succeed");
    assert!(
        report.passed || !report.violations.is_empty(),
        "report should have a definitive result"
    );
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_validate_with_specific_validator() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = workspace_root()?;
    let service = validation_service()?;

    let result = std::thread::Builder::new()
        .name("validate-specific".into())
        .stack_size(8 * 1024 * 1024)
        .spawn(move || {
            tokio::runtime::Runtime::new()
                .map_err(|e| mcb_domain::error::Error::internal(e.to_string()))
                .and_then(|rt| {
                    rt.block_on(service.validate(
                        &workspace_root,
                        Some(&["quality".to_owned()]),
                        Some("warning"),
                    ))
                })
        })?
        .join()
        .map_err(|_| "thread panicked")?;

    let report = result?;
    assert!(report.passed || !report.violations.is_empty());
    Ok(())
}

#[rstest]
#[tokio::test]
async fn test_validate_detects_inline_tests_in_src_via_registry_path()
-> Result<(), Box<dyn std::error::Error>> {
    let tmp = tempfile::tempdir()?;
    let workspace = tmp.path();

    fs::create_dir_all(workspace.join("crates/foo/src"))?;
    fs::write(
        workspace.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/foo\"]\n",
    )?;
    fs::write(
        workspace.join("crates/foo/Cargo.toml"),
        "[package]\nname = \"foo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    fs::write(
        workspace.join("crates/foo/src/lib.rs"),
        "#[cfg(test)]\nmod tests {\n    #[test]\n    fn smoke() {\n        assert_eq!(1, 1);\n    }\n}\n",
    )?;

    let service = validation_service()?;
    let report = service
        .validate(workspace, Some(&["hygiene".to_owned()]), Some("warning"))
        .await?;

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
    Ok(())
}
