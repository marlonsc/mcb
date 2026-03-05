//! Integration tests for CA016/CA018/CA019 enforcement rules.
//!
//! These rules ensure:
//! - CA016: Constants SSOT — no `pub mod constants;` outside mcb-utils
//! - CA018: No proxy/wrapper — no `pub use mcb_utils::` re-exports
//! - CA019: Outer crate isolation — no cross-imports between outer crates

use std::path::PathBuf;

use mcb_domain::ports::validation::ValidationConfig;
use mcb_domain::ports::validation::Validator;
use mcb_validate::validators::declarative_validator::DeclarativeValidator;
use rstest::rstest;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .ancestors()
        .nth(2)
        .map(std::path::Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

/// CA016: No `pub mod constants;` outside mcb-utils in the real workspace.
#[rstest]
fn test_ca016_zero_constants_module_violations() {
    let root = workspace_root();
    let config = ValidationConfig::new(&root);
    let validator = DeclarativeValidator::new(&root);
    let all_violations = validator
        .validate(&config)
        .expect("validation should succeed");

    let ca016: Vec<_> = all_violations
        .iter()
        .filter(|v| format!("{v}").contains("CA016"))
        .collect();

    assert!(
        ca016.is_empty(),
        "CA016: Found {} constants SSOT violations: {:?}",
        ca016.len(),
        ca016.iter().map(|v| format!("{v}")).collect::<Vec<_>>()
    );
}

/// CA018: No `pub use mcb_utils::` re-exports in the real workspace.
#[rstest]
fn test_ca018_zero_proxy_wrapper_violations() {
    let root = workspace_root();
    let config = ValidationConfig::new(&root);
    let validator = DeclarativeValidator::new(&root);
    let all_violations = validator
        .validate(&config)
        .expect("validation should succeed");

    let ca018: Vec<_> = all_violations
        .iter()
        .filter(|v| format!("{v}").contains("CA018"))
        .collect();

    assert!(
        ca018.is_empty(),
        "CA018: Found {} proxy/wrapper violations: {:?}",
        ca018.len(),
        ca018.iter().map(|v| format!("{v}")).collect::<Vec<_>>()
    );
}

/// CA019: No cross-imports between outer crates in the real workspace.
#[rstest]
fn test_ca019_zero_cross_import_violations() {
    let root = workspace_root();
    let config = ValidationConfig::new(&root);
    let validator = DeclarativeValidator::new(&root);
    let all_violations = validator
        .validate(&config)
        .expect("validation should succeed");

    let ca019: Vec<_> = all_violations
        .iter()
        .filter(|v| format!("{v}").contains("CA019"))
        .collect();

    assert!(
        ca019.is_empty(),
        "CA019: Found {} outer crate cross-import violations: {:?}",
        ca019.len(),
        ca019.iter().map(|v| format!("{v}")).collect::<Vec<_>>()
    );
}
