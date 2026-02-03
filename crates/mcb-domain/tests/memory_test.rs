//! Tests for memory entities (REF003: dedicated test file).

use mcb_domain::entities::memory::{
    MemoryFilter, Observation, ObservationMetadata, ObservationType,
};

#[test]
fn test_observation_type_from_str() {
    assert_eq!("code".parse::<ObservationType>().unwrap(), ObservationType::Code);
    assert_eq!("context".parse::<ObservationType>().unwrap(), ObservationType::Context);
    assert!("unknown".parse::<ObservationType>().is_err());
}

#[test]
fn test_observation_type_as_str() {
    assert_eq!(ObservationType::Code.as_str(), "code");
    assert_eq!(ObservationType::Summary.as_str(), "summary");
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
        observation_type: None,
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
        content: "c".to_string(),
        content_hash: "h".to_string(),
        tags: vec![],
        observation_type: ObservationType::Context,
        metadata: ObservationMetadata::default(),
        created_at: 0,
        embedding_id: None,
    };
    assert_eq!(o.id, "id1");
    assert_eq!(o.content, "c");
}
