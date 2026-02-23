//! ErrorPattern domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::error_pattern;
use mcb_domain::entities::memory::{ErrorPattern, ErrorPatternCategory};

impl From<error_pattern::Model> for ErrorPattern {
    fn from(m: error_pattern::Model) -> Self {
        Self {
            id: m.id,
            project_id: m.project_id,
            pattern_signature: m.pattern_signature,
            description: m.description,
            category: m
                .category
                .parse::<ErrorPatternCategory>()
                .unwrap_or(ErrorPatternCategory::Other),
            solutions: m
                .solutions
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            affected_files: m
                .affected_files
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            tags: m
                .tags
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            occurrence_count: m.occurrence_count,
            first_seen_at: m.first_seen_at,
            last_seen_at: m.last_seen_at,
            embedding_id: m.embedding_id,
        }
    }
}

impl From<ErrorPattern> for error_pattern::ActiveModel {
    fn from(e: ErrorPattern) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            project_id: ActiveValue::Set(e.project_id),
            pattern_signature: ActiveValue::Set(e.pattern_signature),
            description: ActiveValue::Set(e.description),
            category: ActiveValue::Set(e.category.to_string()),
            solutions: ActiveValue::Set(Some(
                serde_json::to_string(&e.solutions).unwrap_or_else(|_| "[]".into()),
            )),
            affected_files: ActiveValue::Set(Some(
                serde_json::to_string(&e.affected_files).unwrap_or_else(|_| "[]".into()),
            )),
            tags: ActiveValue::Set(Some(
                serde_json::to_string(&e.tags).unwrap_or_else(|_| "[]".into()),
            )),
            occurrence_count: ActiveValue::Set(e.occurrence_count),
            first_seen_at: ActiveValue::Set(e.first_seen_at),
            last_seen_at: ActiveValue::Set(e.last_seen_at),
            embedding_id: ActiveValue::Set(e.embedding_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_error_pattern() -> ErrorPattern {
        ErrorPattern {
            id: "ep-001".into(),
            project_id: "proj-001".into(),
            pattern_signature: "E0308: mismatched types".into(),
            description: "Type mismatch in function return".into(),
            category: ErrorPatternCategory::Compilation,
            solutions: vec!["Check return type".into()],
            affected_files: vec!["src/main.rs".into()],
            tags: vec!["rust".into(), "type-error".into()],
            occurrence_count: 5,
            first_seen_at: 1700000000,
            last_seen_at: 1700005000,
            embedding_id: Some("emb-001".into()),
        }
    }

    #[test]
    fn round_trip_error_pattern() {
        let domain = sample_error_pattern();
        let active: error_pattern::ActiveModel = domain.clone().into();

        let model = error_pattern::Model {
            id: active.id.unwrap(),
            project_id: active.project_id.unwrap(),
            pattern_signature: active.pattern_signature.unwrap(),
            description: active.description.unwrap(),
            category: active.category.unwrap(),
            solutions: active.solutions.unwrap(),
            affected_files: active.affected_files.unwrap(),
            tags: active.tags.unwrap(),
            occurrence_count: active.occurrence_count.unwrap(),
            first_seen_at: active.first_seen_at.unwrap(),
            last_seen_at: active.last_seen_at.unwrap(),
            embedding_id: active.embedding_id.unwrap(),
        };

        let back: ErrorPattern = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.category, ErrorPatternCategory::Compilation);
        assert_eq!(back.solutions, domain.solutions);
        assert_eq!(back.tags, domain.tags);
    }
}
