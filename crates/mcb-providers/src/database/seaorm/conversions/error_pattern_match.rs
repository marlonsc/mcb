//! ErrorPatternMatch domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::error_pattern_match;
use mcb_domain::entities::memory::ErrorPatternMatch;

impl From<error_pattern_match::Model> for ErrorPatternMatch {
    fn from(m: error_pattern_match::Model) -> Self {
        Self {
            id: m.id,
            pattern_id: m.pattern_id,
            observation_id: m.observation_id,
            confidence: m.confidence,
            solution_applied: m.solution_applied.map(|v| v as i32),
            resolution_successful: m.resolution_successful.map(|v| v != 0),
            matched_at: m.matched_at,
            resolved_at: m.resolved_at,
        }
    }
}

impl From<ErrorPatternMatch> for error_pattern_match::ActiveModel {
    fn from(e: ErrorPatternMatch) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            pattern_id: ActiveValue::Set(e.pattern_id),
            observation_id: ActiveValue::Set(e.observation_id),
            confidence: ActiveValue::Set(e.confidence),
            solution_applied: ActiveValue::Set(e.solution_applied.map(i64::from)),
            resolution_successful: ActiveValue::Set(e.resolution_successful.map(|b| i64::from(b))),
            matched_at: ActiveValue::Set(e.matched_at),
            resolved_at: ActiveValue::Set(e.resolved_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_error_pattern_match() -> ErrorPatternMatch {
        ErrorPatternMatch {
            id: "epm-001".into(),
            pattern_id: "ep-001".into(),
            observation_id: "obs-001".into(),
            confidence: 85,
            solution_applied: Some(0),
            resolution_successful: Some(true),
            matched_at: 1700000000,
            resolved_at: Some(1700001000),
        }
    }

    #[test]
    fn round_trip_error_pattern_match() {
        let domain = sample_error_pattern_match();
        let active: error_pattern_match::ActiveModel = domain.clone().into();

        let model = error_pattern_match::Model {
            id: active.id.unwrap(),
            pattern_id: active.pattern_id.unwrap(),
            observation_id: active.observation_id.unwrap(),
            confidence: active.confidence.unwrap(),
            solution_applied: active.solution_applied.unwrap(),
            resolution_successful: active.resolution_successful.unwrap(),
            matched_at: active.matched_at.unwrap(),
            resolved_at: active.resolved_at.unwrap(),
        };

        let back: ErrorPatternMatch = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.confidence, 85);
        assert_eq!(back.solution_applied, Some(0));
        assert_eq!(back.resolution_successful, Some(true));
    }

    #[test]
    fn round_trip_unresolved_match() {
        let mut domain = sample_error_pattern_match();
        domain.solution_applied = None;
        domain.resolution_successful = None;
        domain.resolved_at = None;

        let active: error_pattern_match::ActiveModel = domain.clone().into();
        let model = error_pattern_match::Model {
            id: active.id.unwrap(),
            pattern_id: active.pattern_id.unwrap(),
            observation_id: active.observation_id.unwrap(),
            confidence: active.confidence.unwrap(),
            solution_applied: active.solution_applied.unwrap(),
            resolution_successful: active.resolution_successful.unwrap(),
            matched_at: active.matched_at.unwrap(),
            resolved_at: active.resolved_at.unwrap(),
        };

        let back: ErrorPatternMatch = model.into();
        assert_eq!(back.solution_applied, None);
        assert_eq!(back.resolution_successful, None);
        assert_eq!(back.resolved_at, None);
    }
}
