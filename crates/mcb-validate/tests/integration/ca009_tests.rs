//! Integration test for CA009 detection
//!
//! CA009: Infrastructure must NOT import from Application layer (except composition root).
//! The validator skips mcb-infrastructure/src/di/ (composition root), so compliant
//! code has zero CA009 violations.

use mcb_validate::{CleanArchitectureValidator, ValidationConfig};
use std::path::PathBuf;

/// Test that CA009 allows composition root (di/) and flags only non-di imports
#[test]
fn test_ca009_infrastructure_imports_application() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let config = ValidationConfig::new(&root);
    let validator = CleanArchitectureValidator::with_config(config);

    let violations = validator.validate_all().expect("validation should succeed");

    let ca009_violations: Vec<_> = violations
        .iter()
        .filter(|v| format!("{v}").contains("CA009"))
        .collect();

    // Composition root (src/di/) is allowed to import mcb_application. All other
    // mcb-infrastructure src code must not depend on Application layer.
    assert!(
        ca009_violations.is_empty(),
        "CA009: mcb-infrastructure (outside di/) must not import mcb_application. \
         Violations: {:?}",
        ca009_violations
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
    );
}
