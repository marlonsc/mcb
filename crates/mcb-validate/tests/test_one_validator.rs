use mcb_validate::ArchitectureValidator;
use std::path::PathBuf;

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .to_path_buf()
}

#[test]
fn test_dependency_only() {
    println!("Testing dependency validator alone...");
    let workspace_root = get_workspace_root();
    let mut validator = ArchitectureValidator::new(&workspace_root);
    let result = validator.validate_dependencies();
    assert!(result.is_ok(), "Failed: {:?}", result.err());
    println!("PASS");
}
