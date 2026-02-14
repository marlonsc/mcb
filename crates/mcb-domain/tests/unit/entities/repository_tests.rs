use mcb_domain::entities::repository::{Branch, Repository, VcsType};
use rstest::*;

#[rstest]
#[case(VcsType::Git, "git")]
#[case(VcsType::Mercurial, "mercurial")]
#[case(VcsType::Svn, "svn")]
fn vcs_type_as_str(#[case] vcs_type: VcsType, #[case] expected: &str) {
    assert_eq!(vcs_type.as_str(), expected);
}

#[rstest]
#[case("git", Ok(VcsType::Git))]
#[case("mercurial", Ok(VcsType::Mercurial))]
#[case("hg", Ok(VcsType::Mercurial))]
#[case("svn", Ok(VcsType::Svn))]
#[case("subversion", Ok(VcsType::Svn))]
#[case("invalid", Err(()))]
// Case insensitive
#[case("GIT", Ok(VcsType::Git))]
#[case("Mercurial", Ok(VcsType::Mercurial))]
#[case("SVN", Ok(VcsType::Svn))]
fn vcs_type_from_str(#[case] input: &str, #[case] expected: Result<VcsType, ()>) {
    if let Ok(expected_type) = expected {
        assert_eq!(input.parse::<VcsType>(), Ok(expected_type));
    } else {
        assert!(input.parse::<VcsType>().is_err());
    }
}

#[fixture]
fn sample_repo() -> Repository {
    Repository {
        id: "repo-001".to_string(),
        org_id: "org-001".to_string(),
        project_id: "proj-001".to_string(),
        name: "mcb-data-model".to_string(),
        url: "https://github.com/org/mcb-data-model".to_string(),
        local_path: "/home/dev/mcb-data-model".to_string(),
        vcs_type: VcsType::Git,
        created_at: 1000,
        updated_at: 1000,
    }
}

#[rstest]
fn repository_construction(sample_repo: Repository) {
    assert_eq!(sample_repo.id, "repo-001");
    assert_eq!(sample_repo.vcs_type, VcsType::Git);
}

#[rstest]
fn repository_serialization_roundtrip(sample_repo: Repository) {
    let json = serde_json::to_string(&sample_repo).expect("serialize");
    let deserialized: Repository = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, sample_repo.id);
    assert_eq!(deserialized.name, sample_repo.name);
    assert_eq!(deserialized.vcs_type, sample_repo.vcs_type);
}

#[fixture]
fn sample_branch() -> Branch {
    Branch {
        id: "br-001".to_string(),
        repository_id: "repo-001".to_string(),
        name: "main".to_string(),
        is_default: true,
        head_commit: "abc123def456".to_string(),
        upstream: Some("origin/main".to_string()),
        created_at: 1000,
    }
}

#[rstest]
fn branch_construction(sample_branch: Branch) {
    assert_eq!(sample_branch.id, "br-001");
    assert!(sample_branch.is_default);
    assert!(sample_branch.upstream.is_some());
}

#[rstest]
fn branch_without_upstream() {
    let branch = Branch {
        id: "br-002".to_string(),
        repository_id: "repo-001".to_string(),
        name: "feat/local-only".to_string(),
        is_default: false,
        head_commit: "deadbeef".to_string(),
        upstream: None,
        created_at: 2000,
    };
    assert!(!branch.is_default);
    assert!(branch.upstream.is_none());
}

#[rstest]
fn branch_serialization_roundtrip(sample_branch: Branch) {
    let json = serde_json::to_string(&sample_branch).expect("serialize");
    let deserialized: Branch = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, sample_branch.id);
    assert_eq!(deserialized.name, sample_branch.name);
}
