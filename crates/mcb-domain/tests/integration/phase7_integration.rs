//! Tests for MEM-05 (Context Injection) and MEM-06 (VCS Tagging)

#[cfg(test)]
mod phase7_integration_tests {
    use mcb_domain::entities::memory::{MemoryFilter, ObservationMetadata, ObservationType};
    use rstest::*;
    use uuid::Uuid;

    fn metadata_with_context(branch: Option<&str>, commit: Option<&str>) -> ObservationMetadata {
        ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-123".to_string()),
            repo_id: Some("repo-abc".to_string()),
            file_path: Some("src/main.rs".to_string()),
            branch: branch.map(str::to_string),
            commit: commit.map(str::to_string),
            execution: None,
            quality_gate: None,
            origin_context: None,
        }
    }

    #[rstest]
    #[case(Some("feature/memory"), Some("abc123def456"))]
    #[case(None, None)]
    fn mem06_observation_metadata_vcs_fields(
        #[case] branch: Option<&str>,
        #[case] commit: Option<&str>,
    ) {
        let metadata = metadata_with_context(branch, commit);
        assert_eq!(metadata.branch.as_deref(), branch);
        assert_eq!(metadata.commit.as_deref(), commit);
    }

    #[rstest]
    #[case(Some(ObservationType::Decision), Some("main"), Some("xyz789"))]
    #[case(None, Some("feature/x"), Some("abc123"))]
    fn memory_filter_supports_vcs_context(
        #[case] observation_type: Option<ObservationType>,
        #[case] branch: Option<&str>,
        #[case] commit: Option<&str>,
    ) {
        let filter = MemoryFilter {
            id: None,
            project_id: None,
            tags: None,
            r#type: observation_type,
            session_id: Some("sess-123".to_string()),
            parent_session_id: None,
            repo_id: Some("repo-abc".to_string()),
            time_range: None,
            branch: branch.map(str::to_string),
            commit: commit.map(str::to_string),
        };

        assert_eq!(filter.branch.as_deref(), branch);
        assert_eq!(filter.commit.as_deref(), commit);
    }

    #[test]
    fn test_mem05_vcs_bootstrap_context_for_session_start() {
        let metadata1 = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-start".to_string()),
            repo_id: Some("repo-1".to_string()),
            file_path: Some("file1.rs".to_string()),
            branch: Some("main".to_string()),
            commit: Some("commit1".to_string()),
            execution: None,
            quality_gate: None,
            origin_context: None,
        };

        let metadata2 = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-start".to_string()),
            repo_id: Some("repo-1".to_string()),
            file_path: Some("file2.rs".to_string()),
            branch: Some("main".to_string()),
            commit: Some("commit2".to_string()),
            execution: None,
            quality_gate: None,
            origin_context: None,
        };

        assert_eq!(metadata1.branch, metadata2.branch);
        assert!(metadata1.commit.is_some() && metadata2.commit.is_some());
    }

    #[test]
    fn test_memory_filter_creates_vcs_aware_queries() {
        let filter = MemoryFilter {
            id: None,
            project_id: None,
            tags: None,
            r#type: None,
            session_id: Some("sess-123".to_string()),
            parent_session_id: None,
            repo_id: Some("repo-abc".to_string()),
            time_range: None,
            branch: Some("feature/x".to_string()),
            commit: Some("abc123".to_string()),
        };

        assert_eq!(filter.session_id, Some("sess-123".to_string()));
        assert_eq!(filter.repo_id, Some("repo-abc".to_string()));
        assert_eq!(filter.branch, Some("feature/x".to_string()));
        assert_eq!(filter.commit, Some("abc123".to_string()));
    }

    #[test]
    fn test_observation_metadata_serialization() {
        let metadata = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-123".to_string()),
            repo_id: Some("repo-abc".to_string()),
            file_path: Some("src/main.rs".to_string()),
            branch: Some("main".to_string()),
            commit: Some("abc123".to_string()),
            execution: None,
            quality_gate: None,
            origin_context: None,
        };

        let json = serde_json::to_value(&metadata).expect("Serialization failed");

        assert_eq!(json["session_id"], "sess-123");
        assert_eq!(json["branch"], "main");
        assert_eq!(json["commit"], "abc123");
    }
}
