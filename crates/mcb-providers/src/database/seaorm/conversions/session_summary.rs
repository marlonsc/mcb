//! SessionSummary domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::session_summary;
use mcb_domain::entities::memory::{OriginContext, SessionSummary};

impl From<session_summary::Model> for SessionSummary {
    fn from(m: session_summary::Model) -> Self {
        Self {
            id: m.id,
            project_id: m.project_id,
            org_id: m.org_id.unwrap_or_default(),
            session_id: m.session_id,
            topics: m
                .topics
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            decisions: m
                .decisions
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            next_steps: m
                .next_steps
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            key_files: m
                .key_files
                .as_deref()
                .and_then(|s| serde_json::from_str(s).ok())
                .unwrap_or_default(),
            origin_context: m
                .origin_context
                .as_deref()
                .and_then(|s| serde_json::from_str::<OriginContext>(s).ok()),
            created_at: m.created_at,
        }
    }
}

impl From<SessionSummary> for session_summary::ActiveModel {
    fn from(e: SessionSummary) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(Some(e.org_id)),
            project_id: ActiveValue::Set(e.project_id),
            repo_id: ActiveValue::NotSet,
            session_id: ActiveValue::Set(e.session_id),
            topics: ActiveValue::Set(Some(
                serde_json::to_string(&e.topics).unwrap_or_else(|_| "[]".into()),
            )),
            decisions: ActiveValue::Set(Some(
                serde_json::to_string(&e.decisions).unwrap_or_else(|_| "[]".into()),
            )),
            next_steps: ActiveValue::Set(Some(
                serde_json::to_string(&e.next_steps).unwrap_or_else(|_| "[]".into()),
            )),
            key_files: ActiveValue::Set(Some(
                serde_json::to_string(&e.key_files).unwrap_or_else(|_| "[]".into()),
            )),
            origin_context: ActiveValue::Set(
                e.origin_context
                    .as_ref()
                    .and_then(|oc| serde_json::to_string(oc).ok()),
            ),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_session_summary() -> SessionSummary {
        SessionSummary {
            id: "ss-001".into(),
            project_id: "proj-001".into(),
            org_id: "org-001".into(),
            session_id: "ses-001".into(),
            topics: vec!["refactoring".into(), "testing".into()],
            decisions: vec!["Use SeaORM".into()],
            next_steps: vec!["Write migrations".into()],
            key_files: vec!["src/main.rs".into()],
            origin_context: None,
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_session_summary() {
        let domain = sample_session_summary();
        let active: session_summary::ActiveModel = domain.clone().into();

        let model = session_summary::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: active.project_id.unwrap(),
            repo_id: None,
            session_id: active.session_id.unwrap(),
            topics: active.topics.unwrap(),
            decisions: active.decisions.unwrap(),
            next_steps: active.next_steps.unwrap(),
            key_files: active.key_files.unwrap(),
            origin_context: active.origin_context.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: SessionSummary = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.topics, domain.topics);
        assert_eq!(back.decisions, domain.decisions);
    }
}
