//! Tests for VCS entities (REF003: dedicated test file).

use rstest::rstest;
use std::path::PathBuf;

use mcb_domain::entities::vcs::{RepositoryId, VcsBranch, VcsRepository};
use mcb_domain::utils::id;

#[rstest]
#[case("abc123")]
#[case("xyz")]
fn repository_id_construction(#[case] input: &str) {
    let uuid = id::deterministic("repository", input);
    let id = RepositoryId::from_uuid(uuid);
    assert_eq!(id.to_string(), uuid.to_string());
}

#[rstest]
fn test_vcs_repository_has_required_fields() {
    let uuid = id::deterministic("repository", "r1");
    let repo = VcsRepository::new(
        RepositoryId::from_uuid(uuid),
        PathBuf::from("/tmp/repo"),
        "main".to_string(),
        vec!["main".to_string()],
        Some("https://example.com".to_string()),
    );
    assert_eq!(repo.id().to_string(), uuid.to_string());
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
