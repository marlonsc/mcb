//!
//! **Documentation**: [docs/modules/validate.md](../../../../../docs/modules/validate.md)
//!
//! # Test Organization and Quality Validation
//!
//! This module provides the infrastructure for validating test hygiene and quality
//! across the codebase. It ensures that tests are organized according to the
//! project's standards (e.g., unit tests in `tests/unit/`) and that they maintain
//! high quality (e.g., meaningful assertions, no raw unwraps).
//!
//! This validator delegates specific checks to specialized modules in the `hygiene` directory.

/// Test hygiene violation types
use super::violation::HygieneViolation;
crate::create_validator!(
    HygieneValidator,
    "hygiene",
    "Validates test hygiene and quality",
    HygieneViolation,
    [
        super::inline_tests::validate_no_inline_tests,
        super::directory::validate_test_directory_structure,
        super::naming::validate_test_naming,
        super::function_naming::validate_test_function_naming,
        super::quality::validate_test_quality,
    ]
);
