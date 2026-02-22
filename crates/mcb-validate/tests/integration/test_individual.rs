use mcb_validate::{ValidationConfig, ValidatorRegistry};
use std::path::{Path, PathBuf};

fn get_workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

#[test]
fn test_just_dependency() {
    println!("Testing ONLY dependency validator...");
    let workspace_root = get_workspace_root();
    let config = ValidationConfig::new(&workspace_root);
    let registry = ValidatorRegistry::standard_for(&workspace_root);
    let result = registry.validate_named(&config, &["dependency"]);
    println!("Result: {:?}", result.as_ref().map(Vec::len));
    assert!(result.is_ok());
}
