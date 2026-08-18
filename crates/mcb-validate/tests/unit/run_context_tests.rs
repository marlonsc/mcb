use std::process::Command;

use tempfile::TempDir;

use mcb_domain::ports::validation::ValidationConfig;
use mcb_validate::run_context::{FileInventorySource, ValidationRunContext};
use rstest::rstest;

/// A `git` invocation isolated from an inherited git environment.
///
/// `git` resolves `GIT_DIR`, `GIT_WORK_TREE`, `GIT_INDEX_FILE` and
/// `GIT_COMMON_DIR` ahead of `-C`, so a suite running inside a git operation
/// (pre-commit hook, rebase, merge) would otherwise stage into the surrounding
/// repository's index instead of the temporary one under test.
fn git_command() -> Command {
    let mut command = Command::new("git");
    for key in [
        "GIT_DIR",
        "GIT_WORK_TREE",
        "GIT_INDEX_FILE",
        "GIT_OBJECT_DIRECTORY",
        "GIT_COMMON_DIR",
    ] {
        command.env_remove(key);
    }
    command
}

#[rstest]
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
fn git_inventory_uses_git_source_when_repository_exists() {
    let temp = TempDir::new().expect("tempdir");
    let root = temp.path();

    let init = git_command()
        .arg("init")
        .arg(root)
        .output()
        .expect("run git init");
    assert!(
        init.status.success(),
        "git init failed: {}",
        String::from_utf8_lossy(&init.stderr)
    );

    std::fs::create_dir_all(root.join("src")).expect("create src");
    std::fs::write(root.join("src/lib.rs"), "pub fn ok() {}\n").expect("write src");

    let add = git_command()
        .arg("-C")
        .arg(root)
        .arg("add")
        .arg("src/lib.rs")
        .output()
        .expect("run git add");
    assert!(
        add.status.success(),
        "git add failed: {}",
        String::from_utf8_lossy(&add.stderr)
    );

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
