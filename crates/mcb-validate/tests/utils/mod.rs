//! Shared test utilities for mcb-validate tests
//!
//! This module provides common test helpers to avoid duplication across test
//! files. It is the single source of truth for:
//!
//! - **Crate creation**:  `create_test_crate`, `create_test_crate_with_file`, etc.
//! - **Fixture loading**: `copy_fixture_crate`, `setup_fixture_workspace`
//! - **DRY helpers**:     `with_fixture_crate`, `with_inline_crate`, `with_fixture_workspace`
//! - **Assertions**:      Re-exported from `mcb_domain::test_assertions`
#![allow(dead_code)]

pub mod test_constants;

// Re-export centralized assertion helpers — single source of truth
pub use mcb_domain::test_assertions::{
    assert_has_violation_matching, assert_no_violation_from_file, assert_no_violations,
    assert_violations_exact,
};

use std::fs;

use tempfile::TempDir;

use self::test_constants::{
    CARGO_TOML_TEMPLATE, CONFIG_FILE_NAME, CRATE_LAYER_MAPPINGS, CRATES_DIR, DEFAULT_VERSION,
    GRL_SIMPLE_RULE, LIB_RS, PROJECT_PREFIX, TEST_WORKSPACE_PATH, WORKSPACE_CARGO_TOML,
};

// ---------------------------------------------------------------------------
// Crate creation utilities (for inline / generated code)
// ---------------------------------------------------------------------------

/// Create a minimal crate structure for testing
pub fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
    create_test_crate_with_file(temp, name, LIB_RS, content);
}

/// Create a crate structure with a specific file name
///
/// # Panics
/// Panics if filesystem operations fail (test-only helper).
pub fn create_test_crate_with_file(temp: &TempDir, name: &str, file_name: &str, content: &str) {
    // Create workspace Cargo.toml if it doesn't exist
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        let write_result = fs::write(&workspace_cargo, WORKSPACE_CARGO_TOML);
        assert!(
            write_result.is_ok(),
            "write workspace Cargo.toml failed: {:?}",
            write_result.err()
        );
    }

    // Create crate structure
    let crate_dir = temp.path().join(CRATES_DIR).join(name).join("src");
    let create_dir_result = fs::create_dir_all(&crate_dir);
    assert!(
        create_dir_result.is_ok(),
        "create crate src dir failed: {:?}",
        create_dir_result.err()
    );
    let write_source_result = fs::write(crate_dir.join(file_name), content);
    assert!(
        write_source_result.is_ok(),
        "write source file failed: {:?}",
        write_source_result.err()
    );

    let cargo_dir = temp.path().join(CRATES_DIR).join(name);
    let write_cargo_result = fs::write(
        cargo_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "{DEFAULT_VERSION}"
"#
        ),
    );
    assert!(
        write_cargo_result.is_ok(),
        "write crate Cargo.toml failed: {:?}",
        write_cargo_result.err()
    );
}

/// Get the workspace root for integration tests.
///
/// Thin wrapper around [`mcb_domain::test_utils::workspace_root`] that panics
/// instead of returning a `Result` — for backward compatibility with callers.
///
/// # Panics
/// Panics if `CARGO_MANIFEST_DIR` has fewer than two parent directories.
#[must_use]
pub fn get_workspace_root() -> std::path::PathBuf {
    mcb_domain::test_utils::workspace_root()
        .expect("workspace root should be discoverable from CARGO_MANIFEST_DIR")
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

// NOTE: assert_no_violations, assert_has_violation_matching, assert_no_violation_from_file,
// and assert_violations_exact are now re-exported from mcb_domain::test_assertions above.

// ---------------------------------------------------------------------------
// Fixture loading utilities
// ---------------------------------------------------------------------------

/// Returns the path to the `tests/fixtures/rust/` directory.
#[must_use]
pub fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust")
}

/// Copies an entire fixture crate directory into the temp workspace.
///
/// This recursively copies `tests/fixtures/rust/crates/{crate_name}/` into
/// `{temp}/crates/{crate_name}/`, preserving the full directory structure
/// including Cargo.toml, src/**, and tests/**.
///
/// Useful for testing validators against realistic multi-file crate sources
/// that contain hard-to-find violations spread across multiple files.
///
/// # Panics
/// Panics if filesystem operations fail or the fixture crate directory does
/// not exist.
pub fn copy_fixture_crate(temp: &TempDir, crate_name: &str) {
    // Ensure workspace Cargo.toml exists
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        let fixture_workspace = fixtures_dir().join("Cargo.toml");
        if fixture_workspace.exists() {
            let copy_workspace_result = fs::copy(&fixture_workspace, &workspace_cargo);
            assert!(
                copy_workspace_result.is_ok(),
                "copy fixture workspace Cargo.toml failed: {:?}",
                copy_workspace_result.err()
            );
        } else {
            let write_workspace_result = fs::write(&workspace_cargo, WORKSPACE_CARGO_TOML);
            assert!(
                write_workspace_result.is_ok(),
                "write workspace Cargo.toml failed: {:?}",
                write_workspace_result.err()
            );
        }
    }

    // Config file
    let config_dest = temp.path().join(CONFIG_FILE_NAME);
    if !config_dest.exists() {
        let fixture_config = fixtures_dir().join(CONFIG_FILE_NAME);
        if fixture_config.exists() {
            let copy_config_result = fs::copy(&fixture_config, &config_dest);
            assert!(
                copy_config_result.is_ok(),
                "copy fixture config failed: {:?}",
                copy_config_result.err()
            );
        }
    }

    let src = fixtures_dir().join(CRATES_DIR).join(crate_name);
    let dst = temp.path().join(CRATES_DIR).join(crate_name);

    assert!(
        src.exists(),
        "Fixture crate '{}' not found at {}",
        crate_name,
        src.display()
    );

    copy_dir_recursive(&src, &dst);
}

/// Recursively copies a directory tree.
fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) {
    let create_dir_result = fs::create_dir_all(dst);
    assert!(
        create_dir_result.is_ok(),
        "create destination directory failed: {:?}",
        create_dir_result.err()
    );
    let read_dir_result = fs::read_dir(src);
    assert!(
        read_dir_result.is_ok(),
        "read source directory failed: {:?}",
        read_dir_result.err()
    );
    if let Ok(entries) = read_dir_result {
        for entry_result in entries {
            assert!(
                entry_result.is_ok(),
                "read directory entry failed: {:?}",
                entry_result.err()
            );
            if let Ok(entry) = entry_result {
                let src_path = entry.path();
                let dst_path = dst.join(entry.file_name());
                if src_path.is_dir() {
                    copy_dir_recursive(&src_path, &dst_path);
                } else {
                    let copy_file_result = fs::copy(&src_path, &dst_path);
                    assert!(
                        copy_file_result.is_ok(),
                        "copy file failed: {:?}",
                        copy_file_result.err()
                    );
                }
            }
        }
    }
}

/// Sets up a temp workspace with multiple fixture crates copied in.
///
/// This is the recommended way to set up multi-crate integration tests:
/// ```rust,ignore
/// let temp = TempDir::new().unwrap();
/// setup_fixture_workspace(&temp, &[DOMAIN_CRATE, SERVER_CRATE, INFRA_CRATE]);
/// // Now run your validator against temp.path()
/// ```
pub fn setup_fixture_workspace(temp: &TempDir, crate_names: &[&str]) {
    for name in crate_names {
        copy_fixture_crate(temp, name);
    }
}

// ---------------------------------------------------------------------------
// DRY test setup helpers — eliminate TempDir/copy/validator boilerplate
// ---------------------------------------------------------------------------

/// Sets up a temp workspace with a single fixture crate copied in.
///
/// Returns `(TempDir, workspace_path)` — the caller creates the validator
/// with `workspace_path`. Keeps ownership of `TempDir` alive for the test.
///
/// # Example
/// ```rust,ignore
/// let (temp, root) = with_fixture_crate(TEST_CRATE);
/// let validator = QualityValidator::new(&root);
/// ```
///
/// # Panics
/// Panics if `TempDir` creation or fixture copy fails.
#[must_use]
pub fn with_fixture_crate(crate_name: &str) -> (TempDir, std::path::PathBuf) {
    let temp_result = TempDir::new();
    assert!(
        temp_result.is_ok(),
        "create temp dir failed: {:?}",
        temp_result.err()
    );
    let temp = temp_result.unwrap_or_else(|_| {
        // assert above guarantees Ok, so this branch is unreachable
        std::process::abort()
    });
    copy_fixture_crate(&temp, crate_name);
    let root = temp.path().to_path_buf();
    (temp, root)
}

/// Sets up a temp workspace with multiple fixture crates copied in.
///
/// Returns `(TempDir, workspace_path)`.
///
/// # Example
/// ```rust,ignore
/// let (temp, root) = with_fixture_workspace(&[DOMAIN_CRATE, SERVER_CRATE]);
/// let validator = RefactoringValidator::new(&root);
/// ```
///
/// # Panics
/// Panics if `TempDir` creation or fixture copy fails.
#[must_use]
pub fn with_fixture_workspace(crate_names: &[&str]) -> (TempDir, std::path::PathBuf) {
    let temp_result = TempDir::new();
    assert!(
        temp_result.is_ok(),
        "create temp dir failed: {:?}",
        temp_result.err()
    );
    let temp = temp_result.unwrap_or_else(|_| unreachable!("assert above guarantees Ok"));
    setup_fixture_workspace(&temp, crate_names);
    let root = temp.path().to_path_buf();
    (temp, root)
}

/// Sets up a temp workspace with a single inline crate (generated code).
///
/// Use this for negative tests or generated content that doesn't make
/// sense as a fixture file.
///
/// # Example
/// ```rust,ignore
/// let (temp, root) = with_inline_crate(TEST_CRATE, "pub fn clean() {}");
/// let validator = QualityValidator::new(&root);
/// ```
///
/// # Panics
/// Panics if `TempDir` creation fails.
#[must_use]
pub fn with_inline_crate(crate_name: &str, content: &str) -> (TempDir, std::path::PathBuf) {
    let temp_result = TempDir::new();
    assert!(
        temp_result.is_ok(),
        "create temp dir failed: {:?}",
        temp_result.err()
    );
    let temp = temp_result.unwrap_or_else(|_| unreachable!("assert above guarantees Ok"));
    create_test_crate(&temp, crate_name, content);
    let root = temp.path().to_path_buf();
    (temp, root)
}

// ===========================================================================
// Engine test helpers — shared across expression, rete, cargo, architecture
// ===========================================================================

/// Creates a minimal `RuleContext` with empty file contents.
///
/// Shared by `cargo_dependency_tests` and any test that needs a bare context.
///
/// # Example
/// ```rust,ignore
/// let ctx = create_rule_context();
/// let result = engine.execute(&rule, &ctx);
/// ```
#[must_use]
pub fn create_rule_context() -> mcb_validate::engines::RuleContext {
    use std::collections::HashMap;
    use std::path::PathBuf;

    mcb_validate::engines::RuleContext {
        workspace_root: PathBuf::from(TEST_WORKSPACE_PATH),
        config: mcb_validate::ValidationConfig::new(TEST_WORKSPACE_PATH),
        ast_data: HashMap::new(),
        cargo_data: HashMap::new(),
        file_contents: HashMap::new(),
        facts: std::sync::Arc::new(Vec::new()),
        graph: std::sync::Arc::new(mcb_validate::graph::DependencyGraph::new()),
    }
}

/// Creates a `RuleContext` pre-loaded with given file entries.
///
/// Shared by `expression_engine_tests` which needs `file_contents` populated.
///
/// # Example
/// ```rust,ignore
/// let ctx = create_rule_context_with_files(&[
///     ("src/main.rs", SNIPPET_MAIN_RS),
///     ("src/lib.rs", SNIPPET_LIB_RS),
/// ]);
/// ```
#[must_use]
pub fn create_rule_context_with_files(
    files: &[(&str, &str)],
) -> mcb_validate::engines::RuleContext {
    let mut ctx = create_rule_context();
    for (path, content) in files {
        ctx.file_contents
            .insert(path.to_string(), content.to_string());
    }
    ctx
}

/// Builds the YAML variable mapping required by `YamlRuleLoader`.
///
/// Populates `project_prefix` and all `{key}_crate` / `{key}_module` entries
/// from [`CRATE_LAYER_MAPPINGS`].
///
/// # Example
/// ```rust,ignore
/// let vars = build_yaml_variables();
/// let loader = YamlRuleLoader::with_variables(rules_dir, Some(vars)).unwrap();
/// ```
#[must_use]
pub fn build_yaml_variables() -> serde_yaml::Value {
    let mut variables = serde_yaml::Mapping::new();
    variables.insert(
        serde_yaml::Value::String("project_prefix".to_owned()),
        serde_yaml::Value::String(PROJECT_PREFIX.to_owned()),
    );

    for &(key, crate_name, module_name) in CRATE_LAYER_MAPPINGS {
        variables.insert(
            serde_yaml::Value::String(format!("{key}_crate")),
            serde_yaml::Value::String(crate_name.to_owned()),
        );
        variables.insert(
            serde_yaml::Value::String(format!("{key}_module")),
            serde_yaml::Value::String(module_name.to_owned()),
        );
    }

    serde_yaml::Value::Mapping(variables)
}

/// Builds a GRL rule string from name, condition, and action.
///
/// # Example
/// ```rust,ignore
/// let grl = build_grl_rule("TestRule", "Facts.x == true", "Facts.y = true");
/// ```
#[must_use]
pub fn build_grl_rule(name: &str, condition: &str, action: &str) -> String {
    GRL_SIMPLE_RULE
        .replace("{name}", name)
        .replace("{condition}", condition)
        .replace("{action}", action)
}

/// Parses a GRL string and returns a populated `KnowledgeBase`.
///
/// # Errors
/// Returns an error if GRL parsing or rule addition fails.
///
/// # Example
/// ```rust,ignore
/// let grl = build_grl_rule("Test", "Facts.x == true", "Facts.y = true");
/// let kb = build_kb_from_grl("test", &grl)?;
/// let mut engine = RustRuleEngine::new(kb);
/// ```
pub fn build_kb_from_grl(
    kb_name: &str,
    grl: &str,
) -> Result<rust_rule_engine::KnowledgeBase, Box<dyn std::error::Error>> {
    use rust_rule_engine::{GRLParser, KnowledgeBase};

    let kb = KnowledgeBase::new(kb_name);
    let rules = GRLParser::parse_rules(grl)?;
    for rule in rules {
        kb.add_rule(rule)?;
    }
    Ok(kb)
}

/// Creates `Facts` pre-populated with the given key-value pairs.
///
/// # Example
/// ```rust,ignore
/// let facts = create_facts(&[
///     (FACT_HAS_INTERNAL_DEPS, RreValue::Boolean(true)),
///     (FACT_VIOLATION_TRIGGERED, RreValue::Boolean(false)),
/// ]);
/// ```
#[must_use]
pub fn create_facts(entries: &[(&str, rust_rule_engine::Value)]) -> rust_rule_engine::Facts {
    let facts = rust_rule_engine::Facts::new();
    for (key, value) in entries {
        facts.set(key, value.clone());
    }
    facts
}

/// Generates a `Cargo.toml` string with the given crate name and dependencies.
///
/// # Example
/// ```rust,ignore
/// let toml = cargo_toml_with_deps("my-crate", &[
///     ("serde", "1.0"),
///     ("my-infrastructure", "0.1.0"),
/// ]);
/// ```
#[must_use]
pub fn cargo_toml_with_deps(crate_name: &str, deps: &[(&str, &str)]) -> String {
    let deps_str: Vec<String> = deps
        .iter()
        .map(|(name, version)| format!("{name} = \"{version}\""))
        .collect();

    CARGO_TOML_TEMPLATE
        .replace("{crate_name}", crate_name)
        .replace("{version}", DEFAULT_VERSION)
        .replace("{deps}", &deps_str.join("\n"))
}
