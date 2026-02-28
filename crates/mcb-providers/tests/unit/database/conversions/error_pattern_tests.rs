//! Tests for error_pattern conversion.

use mcb_domain::entities::memory::ErrorPattern;
use mcb_providers::database::seaorm::entities::error_pattern;

fn sample_error_pattern() -> error_pattern::Model {
    error_pattern::Model {
        id: "error_pattern_test_001".into(),
        project_id: "ref_project_id_001".into(),
        pattern_signature: "test_pattern_signature".into(),
        description: "test_description".into(),
        category: "Other".into(),
        solutions: Some(r#"["tag1","tag2"]"#.into()),
        affected_files: Some(r#"["tag1","tag2"]"#.into()),
        tags: Some(r#"["tag1","tag2"]"#.into()),
        occurrence_count: 5,
        first_seen_at: 1_700_000_000,
        last_seen_at: 1_700_000_000,
        embedding_id: Some("test_embedding_id".into()),
    }
}

#[test]
fn round_trip_error_pattern() {
    let model = sample_error_pattern();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ErrorPattern = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: error_pattern::ActiveModel = domain.into();
}
