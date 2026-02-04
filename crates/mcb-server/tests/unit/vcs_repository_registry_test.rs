use mcb_server::vcs_repository_registry;

#[test]
fn test_record_and_lookup_repository() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let repo_path = temp_dir.path();
    let repo_id = "test-repo-record-lookup";

    let recorded = vcs_repository_registry::record_repository(repo_id, repo_path);
    assert!(recorded.is_ok());

    let resolved = vcs_repository_registry::lookup_repository_path(repo_id);
    assert!(resolved.is_ok());
    assert_eq!(resolved.unwrap(), repo_path);
}

#[test]
fn test_lookup_unknown_repository_fails() {
    let result = vcs_repository_registry::lookup_repository_path("nonexistent-repo-xyz-unique-987");
    assert!(result.is_err());
}
