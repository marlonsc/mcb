use mcb_domain::entities::repository::{Branch, Repository, VcsType};

#[test]
fn vcs_type_as_str() {
    assert_eq!(VcsType::Git.as_str(), "git");
    assert_eq!(VcsType::Mercurial.as_str(), "mercurial");
    assert_eq!(VcsType::Svn.as_str(), "svn");
}

#[test]
fn vcs_type_from_str() {
    assert_eq!("git".parse::<VcsType>(), Ok(VcsType::Git));
    assert_eq!("mercurial".parse::<VcsType>(), Ok(VcsType::Mercurial));
    assert_eq!("hg".parse::<VcsType>(), Ok(VcsType::Mercurial));
    assert_eq!("svn".parse::<VcsType>(), Ok(VcsType::Svn));
    assert_eq!("subversion".parse::<VcsType>(), Ok(VcsType::Svn));
    assert!("invalid".parse::<VcsType>().is_err());
}

#[test]
fn vcs_type_from_str_case_insensitive() {
    assert_eq!("GIT".parse::<VcsType>(), Ok(VcsType::Git));
    assert_eq!("Mercurial".parse::<VcsType>(), Ok(VcsType::Mercurial));
    assert_eq!("SVN".parse::<VcsType>(), Ok(VcsType::Svn));
}

#[test]
fn repository_construction() {
    let repo = Repository {
        id: "repo-001".to_string(),
        org_id: "org-001".to_string(),
        project_id: "proj-001".to_string(),
        name: "mcb-data-model".to_string(),
        url: "https://github.com/org/mcb-data-model".to_string(),
        local_path: "/home/dev/mcb-data-model".to_string(),
        vcs_type: VcsType::Git,
        created_at: 1000,
        updated_at: 1000,
    };
    assert_eq!(repo.id, "repo-001");
    assert_eq!(repo.org_id, "org-001");
    assert_eq!(repo.project_id, "proj-001");
    assert_eq!(repo.name, "mcb-data-model");
    assert_eq!(repo.vcs_type, VcsType::Git);
}

#[test]
fn repository_serialization_roundtrip() {
    let repo = Repository {
        id: "repo-002".to_string(),
        org_id: "org-001".to_string(),
        project_id: "proj-001".to_string(),
        name: "backend".to_string(),
        url: "https://github.com/org/backend".to_string(),
        local_path: "/tmp/backend".to_string(),
        vcs_type: VcsType::Git,
        created_at: 2000,
        updated_at: 3000,
    };
    let json = serde_json::to_string(&repo).expect("serialize");
    let deserialized: Repository = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "repo-002");
    assert_eq!(deserialized.name, "backend");
    assert_eq!(deserialized.url, "https://github.com/org/backend");
    assert_eq!(deserialized.created_at, 2000);
    assert_eq!(deserialized.updated_at, 3000);
}

#[test]
fn branch_construction() {
    let branch = Branch {
        id: "br-001".to_string(),
        repository_id: "repo-001".to_string(),
        name: "main".to_string(),
        is_default: true,
        head_commit: "abc123def456".to_string(),
        upstream: Some("origin/main".to_string()),
        created_at: 1000,
    };
    assert_eq!(branch.id, "br-001");
    assert_eq!(branch.repository_id, "repo-001");
    assert_eq!(branch.name, "main");
    assert!(branch.is_default);
    assert_eq!(branch.upstream, Some("origin/main".to_string()));
}

#[test]
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

#[test]
fn branch_serialization_roundtrip() {
    let branch = Branch {
        id: "br-003".to_string(),
        repository_id: "repo-002".to_string(),
        name: "develop".to_string(),
        is_default: false,
        head_commit: "cafe0123".to_string(),
        upstream: Some("origin/develop".to_string()),
        created_at: 3000,
    };
    let json = serde_json::to_string(&branch).expect("serialize");
    let deserialized: Branch = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "br-003");
    assert_eq!(deserialized.name, "develop");
    assert_eq!(deserialized.head_commit, "cafe0123");
}
