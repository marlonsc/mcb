//! Tests for VCS entities (REF003: dedicated test file).

use mcb_domain::entities::vcs::{RepositoryId, VcsBranch, VcsRepository};
use std::path::PathBuf;

#[test]
fn test_repository_id_new_and_as_str() {
    let id = RepositoryId::new("abc123".to_string());
    assert_eq!(id.as_str(), "abc123");
}

#[test]
fn test_repository_id_from_str() {
    let id: RepositoryId = "xyz".into();
    assert_eq!(id.as_str(), "xyz");
}

#[test]
fn test_vcs_repository_has_required_fields() {
    let repo = VcsRepository {
        id: RepositoryId::new("r1".to_string()),
        path: PathBuf::from("/tmp/repo"),
        default_branch: "main".to_string(),
        branches: vec!["main".to_string()],
        remote_url: Some("https://example.com".to_string()),
    };
    assert_eq!(repo.id.as_str(), "r1");
    assert_eq!(repo.default_branch, "main");
}

#[test]
fn test_vcs_branch_has_id_and_name() {
    let branch = VcsBranch {
        id: "b1".to_string(),
        name: "feature".to_string(),
        head_commit: "c1".to_string(),
        is_default: false,
        upstream: None,
    };
    assert_eq!(branch.id, "b1");
    assert_eq!(branch.name, "feature");
}
