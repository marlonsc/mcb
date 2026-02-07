//! Shared test utilities for mcb-validate tests
//!
//! This module provides common test helpers to avoid duplication across test
//! files. It is the single source of truth for:
//!
//! - **Crate creation**:  `create_test_crate`, `create_test_crate_with_file`, etc.
//! - **Fixture loading**: `copy_fixture_crate`, `setup_fixture_workspace`
//! - **DRY helpers**:     `with_fixture_crate`, `with_inline_crate`, `with_fixture_workspace`
//! - **Assertions**:      `assert_no_violations`, `assert_min_violations`,
//!                        `assert_has_violation_matching`, `assert_no_violation_from_file`
#![allow(dead_code)]

use std::fs;

use tempfile::TempDir;

// ---------------------------------------------------------------------------
// Crate creation utilities (for inline / generated code)
// ---------------------------------------------------------------------------

/// Create a minimal crate structure for testing
pub fn create_test_crate(temp: &TempDir, name: &str, content: &str) {
    create_test_crate_with_file(temp, name, "lib.rs", content);
}

/// Create a crate structure with a specific file name
pub fn create_test_crate_with_file(temp: &TempDir, name: &str, file_name: &str, content: &str) {
    // Create workspace Cargo.toml if it doesn't exist
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        fs::write(
            &workspace_cargo,
            r#"[workspace]
members = ["crates/*"]
"#,
        )
        .unwrap();
    }

    // Create crate structure
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join(file_name), content).unwrap();

    let cargo_dir = temp.path().join("crates").join(name);
    fs::write(
        cargo_dir.join("Cargo.toml"),
        format!(
            r#"[package]
name = "{name}"
version = "0.1.1"
"#
        ),
    )
    .unwrap();
}

/// Create a crate with an additional file (not just lib.rs)
pub fn create_test_crate_with_extra_file(
    temp: &TempDir,
    name: &str,
    lib_content: &str,
    extra_file: &str,
    extra_content: &str,
) {
    create_test_crate(temp, name, lib_content);

    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::write(crate_dir.join(extra_file), extra_content).unwrap();
}

/// Create a crate structure with tests directory
pub fn create_test_crate_with_tests(
    temp: &TempDir,
    name: &str,
    lib_content: &str,
    test_content: &str,
) {
    create_test_crate(temp, name, lib_content);

    let tests_dir = temp.path().join("crates").join(name).join("tests");
    fs::create_dir_all(&tests_dir).unwrap();
    fs::write(tests_dir.join("integration_test.rs"), test_content).unwrap();
}

/// Create a file at a specific path within the temp directory
pub fn create_file_at_path(temp: &TempDir, relative_path: &str, content: &str) {
    let full_path = temp.path().join(relative_path);
    if let Some(parent) = full_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(full_path, content).unwrap();
}

/// Get the workspace root for integration tests
pub fn get_workspace_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Ensure temp directory has workspace structure
pub fn ensure_workspace_structure(temp: &TempDir) {
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        fs::write(
            &workspace_cargo,
            r#"[workspace]
members = ["crates/*"]
"#,
        )
        .unwrap();
    }

    let crates_dir = temp.path().join("crates");
    if !crates_dir.exists() {
        fs::create_dir_all(&crates_dir).unwrap();
    }
}

/// Create a constants.rs file in a test crate (for testing exemptions)
pub fn create_constants_file(temp: &TempDir, name: &str, content: &str) {
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join("constants.rs"), content).unwrap();

    // Ensure Cargo.toml exists
    let cargo_path = temp.path().join("crates").join(name).join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(
            cargo_path,
            format!(
                r#"[package]
name = "{name}"
version = "0.1.1"
"#
            ),
        )
        .unwrap();
    }
}

/// Create a null.rs file in a test crate (for testing null provider exemptions)
pub fn create_null_provider_file(temp: &TempDir, name: &str, content: &str) {
    let crate_dir = temp.path().join("crates").join(name).join("src");
    fs::create_dir_all(&crate_dir).unwrap();
    fs::write(crate_dir.join("null.rs"), content).unwrap();

    // Ensure Cargo.toml exists
    let cargo_path = temp.path().join("crates").join(name).join("Cargo.toml");
    if !cargo_path.exists() {
        fs::write(
            cargo_path,
            format!(
                r#"[package]
name = "{name}"
version = "0.1.1"
"#
            ),
        )
        .unwrap();
    }
}

// ---------------------------------------------------------------------------
// Assertion helpers
// ---------------------------------------------------------------------------

/// Assert that violations list is empty with descriptive message
pub fn assert_no_violations<V: std::fmt::Debug>(violations: &[V], context: &str) {
    assert!(
        violations.is_empty(),
        "{}: expected no violations, got {} - {:?}",
        context,
        violations.len(),
        violations
    );
}

/// Assert that violations list has expected count
pub fn assert_violation_count<V: std::fmt::Debug>(
    violations: &[V],
    expected: usize,
    context: &str,
) {
    assert_eq!(
        violations.len(),
        expected,
        "{}: expected {} violations, got {} - {:?}",
        context,
        expected,
        violations.len(),
        violations
    );
}

/// Assert that violations list has at least a minimum count
pub fn assert_min_violations<V: std::fmt::Debug>(
    violations: &[V],
    min_expected: usize,
    context: &str,
) {
    assert!(
        violations.len() >= min_expected,
        "{}: expected at least {} violations, got {} - {:?}",
        context,
        min_expected,
        violations.len(),
        violations
    );
}

/// Asserts that at least one violation in the list satisfies the predicate.
///
/// Replaces the common pattern:
/// ```rust,ignore
/// let found = violations.iter().any(|v| matches!(v, SomeVariant { .. }));
/// assert!(found, "Expected SomeVariant violation");
/// ```
///
/// # Example
/// ```rust,ignore
/// assert_has_violation_matching(
///     &violations,
///     |v| matches!(v, QualityViolation::FileTooLarge { .. }),
///     "FileTooLarge",
/// );
/// ```
pub fn assert_has_violation_matching<V: std::fmt::Debug>(
    violations: &[V],
    predicate: impl Fn(&V) -> bool,
    violation_name: &str,
) {
    assert!(
        violations.iter().any(predicate),
        "Expected at least one {violation_name} violation, got: {violations:?}"
    );
}

/// Asserts that NO violation in the list matches a file name pattern.
///
/// Useful for testing exemptions (e.g., null.rs, constants.rs should not
/// produce violations).
///
/// # Example
/// ```rust,ignore
/// assert_no_violation_from_file(&violations, "null.rs");
/// ```
pub fn assert_no_violation_from_file<V: std::fmt::Debug>(violations: &[V], file_name: &str) {
    for v in violations {
        let msg = format!("{v:?}");
        assert!(
            !msg.contains(file_name),
            "{file_name} should be exempt from this check: {v:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// Fixture loading utilities
// ---------------------------------------------------------------------------

/// Returns the path to the `tests/fixtures/rust/` directory.
pub fn fixtures_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("rust")
}

/// Loads a fixture `.rs` file by name from `tests/fixtures/rust/`.
///
/// Panics if the fixture file does not exist.
pub fn load_fixture(name: &str) -> String {
    let path = fixtures_dir().join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "Failed to load fixture '{}' from {}: {}",
            name,
            path.display(),
            e
        )
    })
}

/// Sets up a temp workspace with a fixture file placed at the correct
/// crate path expected by validators.
///
/// - `crate_name`: name of the crate directory (e.g. `"my-domain"`)
/// - `src_path`: relative path under `src/` (e.g. `"domain/services/agent.rs"`)
/// - `fixture_name`: name of the `.rs` file in `tests/fixtures/rust/`
///
/// Also copies the crate `Cargo.toml` from fixtures if available, otherwise
/// generates a minimal one.
pub fn setup_fixture_crate(temp: &TempDir, crate_name: &str, src_path: &str, fixture_name: &str) {
    let content = load_fixture(fixture_name);

    // Workspace Cargo.toml
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        let fixture_workspace = fixtures_dir().join("Cargo.toml");
        if fixture_workspace.exists() {
            fs::copy(&fixture_workspace, &workspace_cargo).unwrap();
        } else {
            fs::write(&workspace_cargo, "[workspace]\nmembers = [\"crates/*\"]\n").unwrap();
        }
    }

    // Config file
    let config_dest = temp.path().join("mcb-validate.toml");
    if !config_dest.exists() {
        let fixture_config = fixtures_dir().join("mcb-validate.toml");
        if fixture_config.exists() {
            fs::copy(&fixture_config, &config_dest).unwrap();
        }
    }

    // Crate Cargo.toml
    let crate_dir = temp.path().join("crates").join(crate_name);
    fs::create_dir_all(&crate_dir).unwrap();
    let cargo_dest = crate_dir.join("Cargo.toml");
    if !cargo_dest.exists() {
        let fixture_cargo = fixtures_dir()
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml");
        if fixture_cargo.exists() {
            fs::copy(&fixture_cargo, &cargo_dest).unwrap();
        } else {
            fs::write(
                &cargo_dest,
                format!("[package]\nname = \"{crate_name}\"\nversion = \"0.1.0\"\n"),
            )
            .unwrap();
        }
    }

    // Source file at expected path
    let file_path = crate_dir.join("src").join(src_path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&file_path, content).unwrap();
}

/// Copies an entire fixture crate directory into the temp workspace.
///
/// This recursively copies `tests/fixtures/rust/crates/{crate_name}/` into
/// `{temp}/crates/{crate_name}/`, preserving the full directory structure
/// including Cargo.toml, src/**, and tests/**.
///
/// Useful for testing validators against realistic multi-file crate sources
/// that contain hard-to-find violations spread across multiple files.
pub fn copy_fixture_crate(temp: &TempDir, crate_name: &str) {
    // Ensure workspace Cargo.toml exists
    let workspace_cargo = temp.path().join("Cargo.toml");
    if !workspace_cargo.exists() {
        let fixture_workspace = fixtures_dir().join("Cargo.toml");
        if fixture_workspace.exists() {
            fs::copy(&fixture_workspace, &workspace_cargo).unwrap();
        } else {
            fs::write(&workspace_cargo, "[workspace]\nmembers = [\"crates/*\"]\n").unwrap();
        }
    }

    // Config file
    let config_dest = temp.path().join("mcb-validate.toml");
    if !config_dest.exists() {
        let fixture_config = fixtures_dir().join("mcb-validate.toml");
        if fixture_config.exists() {
            fs::copy(&fixture_config, &config_dest).unwrap();
        }
    }

    let src = fixtures_dir().join("crates").join(crate_name);
    let dst = temp.path().join("crates").join(crate_name);

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
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path);
        } else {
            fs::copy(&src_path, &dst_path).unwrap();
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
pub fn with_fixture_crate(crate_name: &str) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
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
pub fn with_fixture_workspace(crate_names: &[&str]) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
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
pub fn with_inline_crate(crate_name: &str, content: &str) -> (TempDir, std::path::PathBuf) {
    let temp = TempDir::new().unwrap();
    create_test_crate(&temp, crate_name, content);
    let root = temp.path().to_path_buf();
    (temp, root)
}
