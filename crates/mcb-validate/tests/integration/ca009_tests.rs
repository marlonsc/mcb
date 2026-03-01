//! Integration test for CA009 detection
//!
//! CA009: Infrastructure must NOT import from Application layer (except composition root).
//! The validator skips mcb-infrastructure/src/di/ (composition root), so compliant
//! code has zero CA009 violations.

use std::path::PathBuf;

use mcb_validate::config::NamingRulesConfig;
use mcb_validate::{CleanArchitectureValidator, ValidationConfig};
use rstest::rstest;

/// Test that CA009 allows composition root (di/) and flags only non-di imports
#[rstest]
#[test]
fn test_ca009_infrastructure_imports_application() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let config = ValidationConfig::new(&root);
    let file_config = mcb_validate::config::FileConfig::load(&root);
    let validator = CleanArchitectureValidator::with_config(
        &config,
        &file_config.rules.clean_architecture,
        &NamingRulesConfig {
            domain_crate: "mcb-domain".to_owned(),
            application_crate: "mcb-infrastructure".to_owned(),
            providers_crate: "mcb-providers".to_owned(),
            infrastructure_crate: "mcb-infrastructure".to_owned(),
            server_crate: "mcb-server".to_owned(),
            validate_crate: "mcb-validate".to_owned(),

            enabled: true,
        },
    );

    let violations = validator.validate_all().expect("validation should succeed");

    let ca009_violations: Vec<_> = violations
        .iter()
        .filter(|v| format!("{v}").contains("CA009"))
        .collect();

    // Composition root (src/di/) is allowed to import the configured application layer.
    // All other mcb-infrastructure src code must not depend on that layer.
    assert!(
        ca009_violations.is_empty(),
        "CA009: mcb-infrastructure (outside di/) must not import configured application layer. \
         Violations: {:?}",
        ca009_violations
            .iter()
            .map(|v| format!("{v}"))
            .collect::<Vec<_>>()
    );
}
