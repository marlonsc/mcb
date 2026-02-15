//! Integration tests for the validate command.

use mcb::cli::validate::ValidateArgs;
use rstest::*;
use std::fs;

#[fixture]
fn workspace_root() -> std::path::PathBuf {
    let temp_dir = std::env::temp_dir().join(format!("mcb-validate-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("create temp dir");

    // Create mcb.yaml to mark it as a workspace
    fs::write(temp_dir.join("mcb.yaml"), "project: test").expect("write mcb.yaml");

    temp_dir
}

#[fixture]
fn clean_workspace(workspace_root: std::path::PathBuf) -> std::path::PathBuf {
    workspace_root
}

#[rstest]
fn test_validate_execution(clean_workspace: std::path::PathBuf) {
    let args = ValidateArgs {
        path: clean_workspace.clone(),
        quick: true,
        strict: false,
        rules: None,
        validators: None,
        severity: "warning".to_string(),
        format: "text".to_string(),
    };

    let result = args.execute();

    // Cleanup
    let _ = fs::remove_dir_all(&clean_workspace);

    assert!(result.is_ok(), "Validation failed: {:?}", result.err());
    let validation_result = result.unwrap();
    assert_eq!(
        validation_result.errors, 0,
        "Expected 0 errors in empty workspace"
    );
}

#[rstest]
fn test_validate_strict_mode(clean_workspace: std::path::PathBuf) {
    let args = ValidateArgs {
        path: clean_workspace.clone(),
        quick: true,
        strict: true,
        rules: None,
        validators: None,
        severity: "warning".to_string(),
        format: "text".to_string(),
    };

    let result = args.execute();

    // Cleanup
    let _ = fs::remove_dir_all(&clean_workspace);

    assert!(result.is_ok());
    assert!(
        !result.unwrap().failed(),
        "Should pass strict mode in clean workspace"
    );
}
