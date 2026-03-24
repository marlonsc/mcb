//! Tests for memory entities (REF003: dedicated test file).

use mcb_domain::entities::memory::{
    ErrorPattern, ErrorPatternCategory, ErrorPatternMatch, MemoryFilter, Observation,
    ObservationMetadata, ObservationType,
};
use rstest::{fixture, rstest};

#[rstest]
#[case("code", Ok(ObservationType::Code))]
#[case("context", Ok(ObservationType::Context))]
#[case("execution", Ok(ObservationType::Execution))]
#[case("quality_gate", Ok(ObservationType::QualityGate))]
#[case("unknown", Err(()))]
fn observation_type_from_str(#[case] input: &str, #[case] expected: Result<ObservationType, ()>) {
    match expected {
        Ok(observation_type) => assert_eq!(input.parse::<ObservationType>(), Ok(observation_type)),
        Err(()) => assert!(input.parse::<ObservationType>().is_err()),
    }
}

#[rstest]
#[case(ObservationType::Code, "code")]
#[case(ObservationType::Summary, "summary")]
#[case(ObservationType::Execution, "execution")]
#[case(ObservationType::QualityGate, "quality_gate")]
fn observation_type_as_str(#[case] observation_type: ObservationType, #[case] expected: &str) {
    assert_eq!(observation_type.as_str(), expected);
}

#[rstest]
fn test_observation_metadata_default() {
    let m = ObservationMetadata::default();
    assert!(!m.id.is_empty());
    assert!(m.session_id.is_none());
}

#[rstest]
fn test_memory_filter_construction() {
    let f = MemoryFilter {
        id: None,
        project_id: None,
        tags: None,
        r#type: None,
        session_id: Some("s1".to_owned()),
        parent_session_id: None,
        repo_id: None,
        time_range: None,
        branch: None,
        commit: None,
    };
    assert_eq!(f.session_id.as_deref(), Some("s1"));
}

#[fixture]
fn default_observation() -> Observation {
    Observation {
        id: "id1".to_owned(),
        project_id: "test-project".to_owned(),
        content: "c".to_owned(),
        content_hash: "h".to_owned(),
        tags: vec![],
        r#type: ObservationType::Context,
        metadata: ObservationMetadata::default(),
        created_at: 0,
        embedding_id: None,
    }
}

#[rstest]
fn test_observation_has_required_fields(default_observation: Observation) {
    assert_eq!(default_observation.id, "id1");
    assert_eq!(default_observation.content, "c");
}

#[rstest]
#[case("compilation", Ok(ErrorPatternCategory::Compilation))]
#[case("runtime", Ok(ErrorPatternCategory::Runtime))]
#[case("test", Ok(ErrorPatternCategory::Test))]
#[case("lint", Ok(ErrorPatternCategory::Lint))]
#[case("build", Ok(ErrorPatternCategory::Build))]
#[case("config", Ok(ErrorPatternCategory::Config))]
#[case("network", Ok(ErrorPatternCategory::Network))]
#[case("other", Ok(ErrorPatternCategory::Other))]
#[case("invalid", Err(()))]
fn error_pattern_category_from_str(
    #[case] input: &str,
    #[case] expected: Result<ErrorPatternCategory, ()>,
) {
    match expected {
        Ok(category) => assert_eq!(input.parse::<ErrorPatternCategory>(), Ok(category)),
        Err(()) => assert!(input.parse::<ErrorPatternCategory>().is_err()),
    }
}

#[rstest]
#[case(ErrorPatternCategory::Compilation, "compilation")]
#[case(ErrorPatternCategory::Runtime, "runtime")]
#[case(ErrorPatternCategory::Test, "test")]
#[case(ErrorPatternCategory::Lint, "lint")]
#[case(ErrorPatternCategory::Build, "build")]
#[case(ErrorPatternCategory::Config, "config")]
#[case(ErrorPatternCategory::Network, "network")]
#[case(ErrorPatternCategory::Other, "other")]
fn error_pattern_category_as_str(#[case] category: ErrorPatternCategory, #[case] expected: &str) {
    assert_eq!(category.as_str(), expected);
}

#[fixture]
fn default_error_pattern() -> ErrorPattern {
    ErrorPattern {
        id: "ep-001".to_owned(),
        project_id: "proj-1".to_owned(),
        pattern_signature: "error[E0277]: the trait bound".to_owned(),
        description: "Missing trait implementation".to_owned(),
        category: ErrorPatternCategory::Compilation,
        solutions: vec!["Add #[derive(Debug)]".to_owned()],
        affected_files: vec!["src/lib.rs".to_owned()],
        tags: vec!["rust".to_owned(), "trait".to_owned()],
        occurrence_count: 5,
        first_seen_at: 1000,
        last_seen_at: 2000,
        embedding_id: None,
    }
}

#[rstest]
fn test_error_pattern_construction(default_error_pattern: ErrorPattern) {
    assert_eq!(default_error_pattern.id, "ep-001");
    assert_eq!(default_error_pattern.occurrence_count, 5);
    assert_eq!(default_error_pattern.solutions.len(), 1);
}

#[fixture]
fn default_error_pattern_match() -> ErrorPatternMatch {
    ErrorPatternMatch {
        id: "epm-001".to_owned(),
        pattern_id: "ep-001".to_owned(),
        observation_id: "obs-001".to_owned(),
        confidence: 950,
        solution_applied: Some(0),
        resolution_successful: Some(true),
        matched_at: 1500,
        resolved_at: Some(1600),
    }
}

#[rstest]
fn test_error_pattern_match_construction(default_error_pattern_match: ErrorPatternMatch) {
    assert_eq!(default_error_pattern_match.id, "epm-001");
    assert_eq!(default_error_pattern_match.confidence, 950);
    assert!(default_error_pattern_match.resolution_successful.unwrap());
}
