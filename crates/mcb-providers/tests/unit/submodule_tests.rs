use git2::Repository;
use tempfile::TempDir;

use mcb_domain::entities::submodule::SubmoduleInfo;
use mcb_domain::utils::submodule::{collection_name, repo_id};
use mcb_providers::git::submodule::collect_submodules;

#[tokio::test]
async fn test_collect_submodules_empty_repo() {
    let temp = TempDir::new().expect("Failed to create temp dir");
    let repo = Repository::init(temp.path()).expect("Failed to init repo");

    let sig = git2::Signature::now("Test", "test@test.com").unwrap();
    let tree_id = repo.index().unwrap().write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
        .unwrap();

    let result = collect_submodules(temp.path(), "test-repo").await;
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[test]
fn test_submodule_info_collection_name() {
    let info = SubmoduleInfo {
        id: "parent-repo:libs/tree-sitter".to_string(),
        path: "libs/tree-sitter".to_string(),
        url: "https://github.com/tree-sitter/tree-sitter".to_string(),
        commit_hash: "abc123".to_string(),
        parent_repo_id: "parent-repo".to_string(),
        depth: 1,
        name: "tree-sitter".to_string(),
        is_initialized: true,
    };

    assert_eq!(collection_name(&info, "mcb"), "mcb/libs-tree-sitter");
    assert_eq!(repo_id(&info), "parent-repo:libs/tree-sitter");
}
