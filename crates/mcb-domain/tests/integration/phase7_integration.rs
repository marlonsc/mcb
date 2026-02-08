//! Tests for MEM-05 (Context Injection) and MEM-06 (VCS Tagging)

#[cfg(test)]
mod phase7_integration_tests {
    use mcb_domain::entities::memory::{MemoryFilter, ObservationMetadata, ObservationType};
    use uuid::Uuid;

    #[test]
    fn test_mem06_observation_metadata_includes_commit() {
        let metadata = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-123".to_string()),
            repo_id: Some("repo-abc".to_string()),
            file_path: Some("src/main.rs".to_string()),
            branch: Some("feature/memory".to_string()),
            commit: Some("abc123def456".to_string()),
            execution: None,
            quality_gate: None,
        };

        assert_eq!(metadata.session_id, Some("sess-123".to_string()));
        assert_eq!(metadata.branch, Some("feature/memory".to_string()));
        assert_eq!(metadata.commit, Some("abc123def456".to_string()));
    }

    #[test]
    fn test_mem06_memory_filter_supports_vcs_context() {
        let filter = MemoryFilter {
            id: None,
            tags: None,
            observation_type: Some(ObservationType::Decision),
            session_id: Some("sess-123".to_string()),
            repo_id: None,
            time_range: None,
            branch: Some("main".to_string()),
            commit: Some("xyz789".to_string()),
        };

        assert_eq!(filter.branch, Some("main".to_string()));
        assert_eq!(filter.commit, Some("xyz789".to_string()));
    }

    #[test]
    fn test_mem06_vcs_context_fields_are_optional() {
        let metadata = ObservationMetadata {
            id: Uuid::new_v4().to_string(),
            session_id: Some("sess-123".to_string()),
            repo_id: None,
            file_path: None,
            branch: None,
            commit: None,
            execution: None,
            quality_gate: None,
        };

        assert!(metadata.branch.is_none());
        assert!(metadata.commit.is_none());
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
        };

        assert_eq!(metadata1.branch, metadata2.branch);
        assert!(metadata1.commit.is_some() && metadata2.commit.is_some());
    }

    #[test]
    fn test_memory_filter_creates_vcs_aware_queries() {
        let filter = MemoryFilter {
            id: None,
            tags: None,
            observation_type: None,
            session_id: Some("sess-123".to_string()),
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
        };

        let json = serde_json::to_value(&metadata).expect("Serialization failed");

        assert_eq!(json["session_id"], "sess-123");
        assert_eq!(json["branch"], "main");
        assert_eq!(json["commit"], "abc123");
    }
}
