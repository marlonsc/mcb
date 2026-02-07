//! Tests for SOLID Validation
//!
//! Uses fixture crate `my-domain` which contains:
//! - `UserRepository` trait with 8 methods (ISP violation — too many methods)
//! - `InMemoryUserRepo` with todo!()/unimplemented!() stubs (LSP violation)

use mcb_validate::{SolidValidator, SolidViolation};

use crate::test_constants::*;
use crate::test_utils::*;

#[test]
fn test_large_trait_detection() {
    let (_temp, root) = with_fixture_crate(DOMAIN_CRATE);

    // my-domain/src/domain/service.rs has UserRepository with 8 methods
    let validator = SolidValidator::new(&root);
    let violations = validator.validate_isp().unwrap();

    assert_has_violation_matching(
        &violations,
        |v| matches!(v, SolidViolation::TraitTooLarge { .. }),
        "TraitTooLarge for UserRepository",
    );
}

#[test]
fn test_partial_implementation_detection() {
    let (_temp, root) = with_fixture_crate(DOMAIN_CRATE);

    // my-domain/src/domain/service.rs has InMemoryUserRepo with
    // todo!() and unimplemented!() in trait impl methods
    let validator = SolidValidator::new(&root);
    let violations = validator.validate_lsp().unwrap();

    assert_min_violations(
        &violations,
        1,
        "partial trait impl with todo!/unimplemented!",
    );
}

#[test]
fn test_small_trait_no_violation() {
    let (_temp, root) = with_inline_crate(
        TEST_CRATE,
        r"
pub trait SmallTrait {
    fn method_a(&self);
    fn method_b(&self);
}
",
    );

    let validator = SolidValidator::new(&root);
    let violations = validator.validate_isp().unwrap();

    assert_no_violations(&violations, "Small trait should not trigger ISP violation");
}
