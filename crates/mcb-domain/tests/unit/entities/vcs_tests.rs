//! Tests for VCS entities (REF003: dedicated test file).

use rstest::rstest;
use std::path::PathBuf;

use mcb_domain::entities::vcs::{RepositoryId, VcsBranch, VcsRepository};

#[rstest]
#[case("abc123")]
#[case("xyz")]
fn repository_id_construction(#[case] input: &str) {
    let id = RepositoryId::new(input.to_string());
    assert_eq!(id.as_str(), input);

    let from_into: RepositoryId = input.into();
    assert_eq!(from_into.as_str(), input);
}

#[rstest]
fn test_vcs_repository_has_required_fields() {
    let repo = VcsRepository::new(
        RepositoryId::new("r1".to_string()),
        PathBuf::from("/tmp/repo"),
        "main".to_string(),
        vec!["main".to_string()],
        Some("https://example.com".to_string()),
    );
    assert_eq!(repo.id().as_str(), "r1");
    assert_eq!(repo.default_branch(), "main");
}

#[rstest]
fn test_vcs_branch_has_id_and_name() {
    let branch = VcsBranch::new(
        "b1".to_string(),
        "feature".to_string(),
        "c1".to_string(),
        false,
        None,
    );
    assert_eq!(branch.id(), "b1");
    assert_eq!(branch.name(), "feature");
}
