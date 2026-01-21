//! Integration test for CA009 detection
//!
//! This test verifies that the CA009 rule correctly detects
//! when mcb-infrastructure imports from mcb-application.

use mcb_validate::{CleanArchitectureValidator, ValidationConfig};
use std::path::PathBuf;

/// Test that CA009 detects infrastructure importing from application
#[test]
fn test_ca009_detects_infrastructure_imports_application() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    
    let config = ValidationConfig::new(&root);
    let validator = CleanArchitectureValidator::with_config(config);
    
    let violations = validator.validate_all().expect("validation should succeed");
    
    // Filter for CA009 violations
    let ca009_violations: Vec<_> = violations.iter()
        .filter(|v| format!("{}", v).contains("CA009"))
        .collect();
    
    // We expect violations because mcb-infrastructure currently imports from mcb-application
    assert!(
        !ca009_violations.is_empty(),
        "Expected CA009 violations for infrastructure importing application, found none. \
         Check if mcb-infrastructure/src/ has 'use mcb_application' imports."
    );
    
    // Verify the violations are for the correct crate
    for violation in &ca009_violations {
        let msg = format!("{}", violation);
        assert!(
            msg.contains("mcb-infrastructure") || msg.contains("mcb_infrastructure"),
            "CA009 violation should mention mcb-infrastructure: {}",
            msg
        );
    }
    
    println!("=== CA009 Violations Found: {} ===", ca009_violations.len());
    for v in ca009_violations.iter().take(5) {
        println!("{}", v);
    }
    if ca009_violations.len() > 5 {
        println!("... and {} more", ca009_violations.len() - 5);
    }
}
