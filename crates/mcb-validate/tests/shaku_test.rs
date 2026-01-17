//! Tests for DI/Shaku Pattern Validation

mod test_utils;

use mcb_validate::ShakuValidator;
use tempfile::TempDir;
use test_utils::{create_test_crate, create_test_crate_with_file};

#[test]
fn test_direct_instantiation() {
    let temp = TempDir::new().unwrap();
    // Use service.rs instead of lib.rs since lib.rs is skipped by validator
    create_test_crate_with_file(
        &temp,
        "mcb-test",
        "service.rs",
        r#"
pub fn setup() {
    let service = MyService::new();
    let provider = EmbeddingProvider::new();
}
"#,
    );

    let validator = ShakuValidator::new(temp.path());
    let violations = validator.validate_direct_instantiation().unwrap();

    assert_eq!(violations.len(), 2);
}

#[test]
fn test_fake_implementation() {
    let temp = TempDir::new().unwrap();
    // Use service.rs instead of lib.rs since lib.rs is skipped by validator
    create_test_crate_with_file(
        &temp,
        "mcb-test",
        "service.rs",
        r#"
pub fn setup() {
    let provider: Arc<dyn Provider> = Arc::new(NullProvider::new());
    let mock = MockService::new();
}
"#,
    );

    let validator = ShakuValidator::new(temp.path());
    let violations = validator.validate_fake_implementations().unwrap();

    assert_eq!(violations.len(), 2);
}

#[test]
fn test_no_violations_in_tests() {
    let temp = TempDir::new().unwrap();
    create_test_crate(
        &temp,
        "mcb-test",
        r#"
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it() {
        let mock = MockService::new();
        let null = NullProvider::new();
    }
}
"#,
    );

    let validator = ShakuValidator::new(temp.path());
    let violations = validator.validate_all().unwrap();

    assert!(
        violations.is_empty(),
        "Test code should not trigger violations"
    );
}
