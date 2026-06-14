use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use rstest::{fixture, rstest};

#[fixture]
fn sample_repo() -> Repository {
    Repository {
        id: "repo-001".to_owned(),
        org_id: "org-001".to_owned(),
        project_id: "proj-001".to_owned(),
        name: "mcb-data-model".to_owned(),
        url: "https://github.com/org/mcb-data-model".to_owned(),
        local_path: "/home/dev/mcb-data-model".to_owned(),
        vcs_type: VcsType::Git,
        created_at: 1000,
        updated_at: 1000,
    }
}

#[fixture]
fn sample_branch(sample_repo: Repository) -> Branch {
    Branch {
        id: "br-001".to_owned(),
        org_id: sample_repo.org_id,
        repository_id: sample_repo.id,
        name: "main".to_owned(),
        is_default: true,
        head_commit: "abc123def456".to_owned(),
        upstream: Some("origin/main".to_owned()),
        created_at: 1000,
    }
}

#[rstest]
fn test_repository_construction(sample_repo: Repository) {
    assert_eq!(sample_repo.id, "repo-001");
    assert_eq!(sample_repo.vcs_type, VcsType::Git);
}

#[rstest]
fn test_repository_serialization_roundtrip(sample_repo: Repository) {
    let json = serde_json::to_string(&sample_repo).expect("serialize");
    let deserialized: Repository = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, sample_repo.id);
    assert_eq!(deserialized.vcs_type, VcsType::Git);
}

#[rstest]
fn test_branch_construction(sample_branch: Branch) {
    assert_eq!(sample_branch.id, "br-001");
    assert!(sample_branch.is_default);
}

#[rstest]
fn test_branch_without_upstream(mut sample_branch: Branch) {
    sample_branch.is_default = false;
    sample_branch.upstream = None;
    assert!(!sample_branch.is_default);
    assert!(sample_branch.upstream.is_none());
}

#[rstest]
fn test_branch_serialization_roundtrip(sample_branch: Branch) {
    let json = serde_json::to_string(&sample_branch).expect("serialize");
    let deserialized: Branch = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, sample_branch.id);
}

#[rstest]
#[case(VcsType::Git, "git")]
#[case(VcsType::Mercurial, "mercurial")]
#[case(VcsType::Svn, "svn")]
fn test_vcs_type_as_str(#[case] vcs_type: VcsType, #[case] expected: &str) {
    assert_eq!(vcs_type.as_str(), expected);
}

#[rstest]
#[case("git", Ok(VcsType::Git))]
#[case("mercurial", Ok(VcsType::Mercurial))]
#[case("hg", Ok(VcsType::Mercurial))]
#[case("svn", Ok(VcsType::Svn))]
#[case("subversion", Ok(VcsType::Svn))]
#[case("GIT", Ok(VcsType::Git))]
#[case("Mercurial", Ok(VcsType::Mercurial))]
#[case("SVN", Ok(VcsType::Svn))]
#[case("invalid", Err(()))]
fn test_vcs_type_from_str(#[case] input: &str, #[case] expected: Result<VcsType, ()>) {
    if let Ok(expected_type) = expected {
        assert_eq!(input.parse::<VcsType>(), Ok(expected_type));
    } else {
        assert!(input.parse::<VcsType>().is_err());
    }
}
