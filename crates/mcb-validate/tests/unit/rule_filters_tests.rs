//! Unit tests for rule filter executor.

use std::fs;
use std::path::Path;

use mcb_validate::filters::{RuleFilterExecutor, RuleFilters, WorkspaceDependencies};
use tempfile::TempDir;

#[tokio::test]
async fn test_no_filters() {
    let temp_dir = TempDir::new().unwrap();
    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());

    let filters = RuleFilters {
        languages: None,
        dependencies: None,
        file_patterns: None,
    };

    let workspace_deps = WorkspaceDependencies {
        deps: std::collections::HashMap::new(),
    };

    assert!(
        executor
            .should_execute_rule(&filters, Path::new("main.rs"), None, &workspace_deps)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_language_filter() {
    let temp_dir = TempDir::new().unwrap();
    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());

    let filters = RuleFilters {
        languages: Some(vec!["rust".to_string()]),
        dependencies: None,
        file_patterns: None,
    };

    let workspace_deps = WorkspaceDependencies {
        deps: std::collections::HashMap::new(),
    };

    assert!(
        executor
            .should_execute_rule(&filters, Path::new("main.rs"), None, &workspace_deps)
            .await
            .unwrap()
    );

    assert!(
        !executor
            .should_execute_rule(&filters, Path::new("script.py"), None, &workspace_deps)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_dependency_filter() {
    let temp_dir = TempDir::new().unwrap();

    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();

    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());
    let workspace_deps = executor.parse_workspace_dependencies().unwrap();

    let filters = RuleFilters {
        languages: None,
        dependencies: Some(vec!["serde".to_string()]),
        file_patterns: None,
    };

    assert!(
        executor
            .should_execute_rule(
                &filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );

    let tokio_filters = RuleFilters {
        languages: None,
        dependencies: Some(vec!["tokio".to_string()]),
        file_patterns: None,
    };

    assert!(
        !executor
            .should_execute_rule(
                &tokio_filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_file_pattern_filter() {
    let temp_dir = TempDir::new().unwrap();
    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());

    let filters = RuleFilters {
        languages: None,
        dependencies: None,
        file_patterns: Some(vec!["src/**/*.rs".to_string(), "!**/tests/**".to_string()]),
    };

    let workspace_deps = WorkspaceDependencies {
        deps: std::collections::HashMap::new(),
    };

    assert!(
        executor
            .should_execute_rule(&filters, Path::new("src/main.rs"), None, &workspace_deps)
            .await
            .unwrap()
    );

    assert!(
        !executor
            .should_execute_rule(
                &filters,
                Path::new("src/tests/main.rs"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );

    assert!(
        !executor
            .should_execute_rule(&filters, Path::new("lib.py"), None, &workspace_deps)
            .await
            .unwrap()
    );
}

#[tokio::test]
async fn test_combined_filters() {
    let temp_dir = TempDir::new().unwrap();

    let cargo_toml = temp_dir.path().join("Cargo.toml");
    fs::write(
        &cargo_toml,
        r#"
[package]
name = "test"
version = "0.1.0"

[dependencies]
serde = "1.0"
"#,
    )
    .unwrap();

    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());
    let workspace_deps = executor.parse_workspace_dependencies().unwrap();

    let filters = RuleFilters {
        languages: Some(vec!["rust".to_string()]),
        dependencies: Some(vec!["serde".to_string()]),
        file_patterns: Some(vec!["**/src/**/*.rs".to_string()]),
    };

    assert!(
        executor
            .should_execute_rule(
                &filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );

    assert!(
        !executor
            .should_execute_rule(
                &filters,
                &cargo_toml.with_file_name("src/main.py"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );

    let missing_dep_filters = RuleFilters {
        languages: Some(vec!["rust".to_string()]),
        dependencies: Some(vec!["tokio".to_string()]),
        file_patterns: Some(vec!["**/src/**/*.rs".to_string()]),
    };

    assert!(
        !executor
            .should_execute_rule(
                &missing_dep_filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .await
            .unwrap()
    );
}
