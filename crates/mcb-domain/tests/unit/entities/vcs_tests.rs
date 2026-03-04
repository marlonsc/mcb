//! Tests for VCS entities (REF003: dedicated test file).

use rstest::{fixture, rstest};
use std::path::PathBuf;

use mcb_domain::entities::vcs::{VcsBranch, VcsRepository};
use mcb_domain::value_objects::RepositoryId;
use mcb_utils::utils::id;

#[rstest]
#[case("abc123")]
#[case("xyz")]
fn repository_id_construction(#[case] input: &str) {
    let uuid = id::deterministic("repository", input);
    let id = RepositoryId::from_uuid(uuid);
    assert_eq!(id.to_string(), uuid.to_string());
}

#[fixture]
fn vcs_repo() -> VcsRepository {
    let uuid = id::deterministic("repository", "r1");
    VcsRepository::new(
        RepositoryId::from_uuid(uuid),
        PathBuf::from("/tmp/repo"),
        "main".to_owned(),
        vec!["main".to_owned()],
        Some("https://example.com".to_owned()),
    )
}

#[rstest]
fn test_vcs_repository_has_required_fields(vcs_repo: VcsRepository) {
    let uuid = id::deterministic("repository", "r1");
    assert_eq!(vcs_repo.id().to_string(), uuid.to_string());
    assert_eq!(vcs_repo.default_branch(), "main");
}

#[fixture]
fn vcs_branch() -> VcsBranch {
    VcsBranch::new(
        "b1".to_owned(),
        "feature".to_owned(),
        "c1".to_owned(),
        false,
        None,
    )
}

#[rstest]
fn test_vcs_branch_has_id_and_name(vcs_branch: VcsBranch) {
    assert_eq!(vcs_branch.id(), "b1");
    assert_eq!(vcs_branch.name(), "feature");
}
