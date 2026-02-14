use git2::Repository;
use mcb_domain::entities::submodule::SubmoduleInfo;
use mcb_providers::git::submodule::collect_submodules;
use rstest::rstest;
use tempfile::TempDir;

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

#[rstest]
#[case("mcb", "mcb/libs-tree-sitter")]
#[case("project-x", "project-x/libs-tree-sitter")]
fn submodule_info_collection_name(#[case] collection: &str, #[case] expected: &str) {
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

    assert_eq!(info.collection_name(collection), expected);
    assert_eq!(info.repo_id(), "parent-repo:libs/tree-sitter");
}
