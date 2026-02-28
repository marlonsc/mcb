//! Tests for error_pattern_match conversion.

use mcb_domain::entities::memory::ErrorPatternMatch;
use mcb_providers::database::seaorm::entities::error_pattern_match;

fn sample_error_pattern_match() -> error_pattern_match::Model {
    error_pattern_match::Model {
        id: "error_pattern_match_test_001".into(),
        pattern_id: "ref_pattern_id_001".into(),
        observation_id: "ref_observation_id_001".into(),
        confidence: 85,
        solution_applied: Some(3),
        resolution_successful: Some(1),
        matched_at: 1_700_000_000,
        resolved_at: Some(1_700_000_000),
    }
}

#[test]
fn round_trip_error_pattern_match() {
    let model = sample_error_pattern_match();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: ErrorPatternMatch = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: error_pattern_match::ActiveModel = domain.into();
}
