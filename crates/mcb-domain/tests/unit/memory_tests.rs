//! Tests for memory entities (REF003: dedicated test file).

use mcb_domain::entities::memory::{
    ErrorPattern, ErrorPatternCategory, ErrorPatternMatch, MemoryFilter, Observation,
    ObservationMetadata, ObservationType,
};

#[test]
fn test_observation_type_from_str() {
    assert_eq!("code".parse::<ObservationType>(), Ok(ObservationType::Code));
    assert_eq!(
        "context".parse::<ObservationType>(),
        Ok(ObservationType::Context)
    );
    assert_eq!(
        "execution".parse::<ObservationType>(),
        Ok(ObservationType::Execution)
    );
    assert_eq!(
        "quality_gate".parse::<ObservationType>(),
        Ok(ObservationType::QualityGate)
    );
    assert!("unknown".parse::<ObservationType>().is_err());
}

#[test]
fn test_observation_type_as_str() {
    assert_eq!(ObservationType::Code.as_str(), "code");
    assert_eq!(ObservationType::Summary.as_str(), "summary");
    assert_eq!(ObservationType::Execution.as_str(), "execution");
    assert_eq!(ObservationType::QualityGate.as_str(), "quality_gate");
}

#[test]
fn test_observation_metadata_default() {
    let m = ObservationMetadata::default();
    assert!(!m.id.is_empty());
    assert!(m.session_id.is_none());
}

#[test]
fn test_memory_filter_construction() {
    let f = MemoryFilter {
        id: None,
        tags: None,
        r#type: None,
        session_id: Some("s1".to_string()),
        repo_id: None,
        time_range: None,
        branch: None,
        commit: None,
    };
    assert_eq!(f.session_id.as_deref(), Some("s1"));
}

#[test]
fn test_observation_has_required_fields() {
    let o = Observation {
        id: "id1".to_string(),
        project_id: "test-project".to_string(),
        content: "c".to_string(),
        content_hash: "h".to_string(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: ObservationMetadata::default(),
        created_at: 0,
        embedding_id: None,
    };
    assert_eq!(o.id, "id1");
    assert_eq!(o.content, "c");
}

#[test]
fn test_error_pattern_category_from_str() {
    assert_eq!(
        "compilation".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Compilation)
    );
    assert_eq!(
        "runtime".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Runtime)
    );
    assert_eq!(
        "test".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Test)
    );
    assert_eq!(
        "lint".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Lint)
    );
    assert_eq!(
        "build".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Build)
    );
    assert_eq!(
        "config".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Config)
    );
    assert_eq!(
        "network".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Network)
    );
    assert_eq!(
        "other".parse::<ErrorPatternCategory>(),
        Ok(ErrorPatternCategory::Other)
    );
    assert!("invalid".parse::<ErrorPatternCategory>().is_err());
}

#[test]
fn test_error_pattern_category_as_str() {
    assert_eq!(ErrorPatternCategory::Compilation.as_str(), "compilation");
    assert_eq!(ErrorPatternCategory::Runtime.as_str(), "runtime");
    assert_eq!(ErrorPatternCategory::Test.as_str(), "test");
    assert_eq!(ErrorPatternCategory::Lint.as_str(), "lint");
    assert_eq!(ErrorPatternCategory::Build.as_str(), "build");
    assert_eq!(ErrorPatternCategory::Config.as_str(), "config");
    assert_eq!(ErrorPatternCategory::Network.as_str(), "network");
    assert_eq!(ErrorPatternCategory::Other.as_str(), "other");
}

#[test]
fn test_error_pattern_construction() {
    let pattern = ErrorPattern {
        id: "ep-001".to_string(),
        project_id: "proj-1".to_string(),
        pattern_signature: "error[E0277]: the trait bound".to_string(),
        description: "Missing trait implementation".to_string(),
        category: ErrorPatternCategory::Compilation,
        solutions: vec!["Add #[derive(Debug)]".to_string()],
        affected_files: vec!["src/lib.rs".to_string()],
        tags: vec!["rust".to_string(), "trait".to_string()],
        occurrence_count: 5,
        first_seen_at: 1000,
        last_seen_at: 2000,
        embedding_id: None,
    };
    assert_eq!(pattern.id, "ep-001");
    assert_eq!(pattern.occurrence_count, 5);
    assert_eq!(pattern.solutions.len(), 1);
}

#[test]
fn test_error_pattern_match_construction() {
    let match_ = ErrorPatternMatch {
        id: "epm-001".to_string(),
        pattern_id: "ep-001".to_string(),
        observation_id: "obs-001".to_string(),
        confidence: 950,
        solution_applied: Some(0),
        resolution_successful: Some(true),
        matched_at: 1500,
        resolved_at: Some(1600),
    };
    assert_eq!(match_.id, "epm-001");
    assert_eq!(match_.confidence, 950);
    assert!(match_.resolution_successful.unwrap());
}
