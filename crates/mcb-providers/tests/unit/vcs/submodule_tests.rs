use git2::Repository;
use mcb_domain::entities::submodule::SubmoduleInfo;
use mcb_providers::vcs::collect_submodules;
use rstest::rstest;
use tempfile::TempDir;

#[rstest]
#[tokio::test]
async fn test_collect_submodules_empty_repo() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let repo = Repository::init(temp.path())?;

    let sig = git2::Signature::now("Test", "test@test.com")?;
    let mut index = repo.index()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])?;

    let result = collect_submodules(temp.path(), "test-repo").await?;
    assert!(result.is_empty());
    Ok(())
}

#[rstest]
#[case("mcb", "mcb/libs-tree-sitter")]
#[case("project-x", "project-x/libs-tree-sitter")]
fn submodule_info_collection_name(#[case] collection: &str, #[case] expected: &str) {
    let info = SubmoduleInfo {
        id: "parent-repo:libs/tree-sitter".to_owned(),
        path: "libs/tree-sitter".to_owned(),
        url: "https://github.com/tree-sitter/tree-sitter".to_owned(),
        commit_hash: "abc123".to_owned(),
        parent_repo_id: "parent-repo".to_owned(),
        depth: 1,
        name: "tree-sitter".to_owned(),
        is_initialized: true,
    };

    assert_eq!(info.collection_name(collection), expected);
    assert_eq!(info.repo_id(), "parent-repo:libs/tree-sitter");
}
