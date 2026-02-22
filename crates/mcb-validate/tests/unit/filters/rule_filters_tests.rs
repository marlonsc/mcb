//! Unit tests for rule filter executor.

use rstest::rstest;
use std::fs;
use std::path::Path;

use mcb_validate::filters::{RuleFilterExecutor, RuleFilters, WorkspaceDependencies};
use rstest::*;
use tempfile::TempDir;

#[fixture]
fn empty_workspace_deps() -> WorkspaceDependencies {
    WorkspaceDependencies {
        deps: std::collections::HashMap::new(),
    }
}

#[rstest]
#[case(None, "main.rs", true)]
#[case(Some("rust"), "main.rs", true)]
#[case(Some("rust"), "script.py", false)]
#[test]
fn language_filter(
    #[case] language: Option<&str>,
    #[case] file: &str,
    #[case] expected: bool,
    empty_workspace_deps: WorkspaceDependencies,
) {
    let temp_dir = TempDir::new().unwrap();
    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());

    let filters = RuleFilters {
        languages: language.map(|lang| vec![lang.to_owned()]),
        dependencies: None,
        file_patterns: None,
        allow: None,
        deny: None,
        skip: None,
    };

    let actual = executor
        .should_execute_rule(&filters, Path::new(file), None, &empty_workspace_deps)
        .unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_dependency_filter() {
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
        dependencies: Some(vec!["serde".to_owned()]),
        file_patterns: None,
        allow: None,
        deny: None,
        skip: None,
    };

    assert!(
        executor
            .should_execute_rule(
                &filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .unwrap()
    );

    let tokio_filters = RuleFilters {
        languages: None,
        dependencies: Some(vec!["tokio".to_owned()]),
        file_patterns: None,
        allow: None,
        deny: None,
        skip: None,
    };

    assert!(
        !executor
            .should_execute_rule(
                &tokio_filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .unwrap()
    );
}

#[test]
fn test_file_pattern_filter() {
    let temp_dir = TempDir::new().unwrap();
    let executor = RuleFilterExecutor::new(temp_dir.path().to_path_buf());

    let filters = RuleFilters {
        languages: None,
        dependencies: None,
        file_patterns: Some(vec!["src/**/*.rs".to_owned(), "!**/tests/**".to_owned()]),
        allow: None,
        deny: None,
        skip: None,
    };

    let workspace_deps = WorkspaceDependencies {
        deps: std::collections::HashMap::new(),
    };

    assert!(
        executor
            .should_execute_rule(&filters, Path::new("src/main.rs"), None, &workspace_deps)
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
            .unwrap()
    );

    assert!(
        !executor
            .should_execute_rule(&filters, Path::new("lib.py"), None, &workspace_deps)
            .unwrap()
    );
}

#[test]
fn test_combined_filters() {
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
        languages: Some(vec!["rust".to_owned()]),
        dependencies: Some(vec!["serde".to_owned()]),
        file_patterns: Some(vec!["**/src/**/*.rs".to_owned()]),
        allow: None,
        deny: None,
        skip: None,
    };

    assert!(
        executor
            .should_execute_rule(
                &filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
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
            .unwrap()
    );

    let missing_dep_filters = RuleFilters {
        languages: Some(vec!["rust".to_owned()]),
        dependencies: Some(vec!["tokio".to_owned()]),
        file_patterns: Some(vec!["**/src/**/*.rs".to_owned()]),
        allow: None,
        deny: None,
        skip: None,
    };

    assert!(
        !executor
            .should_execute_rule(
                &missing_dep_filters,
                &cargo_toml.with_file_name("src/main.rs"),
                None,
                &workspace_deps
            )
            .unwrap()
    );
}
