use std::process::Command;

use tempfile::TempDir;

use mcb_validate::ValidationConfig;
use mcb_validate::run_context::{FileInventorySource, ValidationRunContext};
use rstest::rstest;

#[rstest]
#[test]
fn walkdir_inventory_respects_exclude_patterns() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::create_dir_all(root.join("target/generated")).expect("create target");
    std::fs::write(root.join("src/lib.rs"), "pub fn ok() {}\n").expect("write src");
    std::fs::write(root.join("target/generated/out.rs"), "pub fn skip() {}\n")
        .expect("write target");

    let config = ValidationConfig::new(root).with_exclude_pattern("target/");
    let context = ValidationRunContext::build(&config).expect("context");

    assert_eq!(
        context.file_inventory_source(),
        FileInventorySource::WalkDir
    );
    assert!(
        context
            .file_inventory()
            .iter()
            .any(|entry| entry.relative_path == std::path::Path::new("src/lib.rs"))
    );
    assert!(context.file_inventory().iter().all(|entry| {
        entry
            .relative_path
            .to_str()
            .is_none_or(|path| !path.contains("target/"))
    }));
}

#[rstest]
#[test]
fn git_inventory_uses_git_source_when_repository_exists() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    let init = Command::new("git")
        .arg("init")
        .arg(root)
        .status()
        .expect("run git init");
    assert!(init.success());

    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/lib.rs"), "pub fn ok() {}\n").expect("write src");

    let add = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("add")
        .arg("src/lib.rs")
        .status()
        .expect("run git add");
    assert!(add.success());

    let config = ValidationConfig::new(root);
    let context = ValidationRunContext::build(&config).expect("context");

    assert_eq!(context.file_inventory_source(), FileInventorySource::Git);
    assert!(
        context
            .file_inventory()
            .iter()
            .any(|entry| entry.relative_path == std::path::Path::new("src/lib.rs"))
    );
}
