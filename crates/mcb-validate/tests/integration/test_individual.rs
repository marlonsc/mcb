use mcb_validate::{ValidationConfig, ValidatorRegistry};
use rstest::rstest;

#[rstest]
#[test]
fn test_just_dependency() {
    println!("Testing ONLY dependency validator...");
    let workspace_root = mcb_domain::utils::tests::utils::workspace_root().unwrap();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let result = registry.validate_named(&config, &["dependency"]);
    println!("Result: {:?}", result.as_ref().map(Vec::len));
    let violations = result.expect("dependency validator should run");
    assert!(violations.iter().all(|v| !v.id().is_empty()));
}
