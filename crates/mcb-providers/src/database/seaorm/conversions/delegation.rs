//! Delegation domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::delegation;
use mcb_domain::entities::Delegation;

impl From<delegation::Model> for Delegation {
    fn from(m: delegation::Model) -> Self {
        Self {
            id: m.id,
            parent_session_id: m.parent_session_id,
            child_session_id: m.child_session_id,
            prompt: m.prompt,
            prompt_embedding_id: m.prompt_embedding_id,
            result: m.result,
            success: m.success != 0,
            created_at: m.created_at,
            completed_at: m.completed_at,
            duration_ms: m.duration_ms,
        }
    }
}

impl From<Delegation> for delegation::ActiveModel {
    fn from(e: Delegation) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            parent_session_id: ActiveValue::Set(e.parent_session_id),
            child_session_id: ActiveValue::Set(e.child_session_id),
            prompt: ActiveValue::Set(e.prompt),
            prompt_embedding_id: ActiveValue::Set(e.prompt_embedding_id),
            result: ActiveValue::Set(e.result),
            success: ActiveValue::Set(i64::from(e.success)),
            created_at: ActiveValue::Set(e.created_at),
            completed_at: ActiveValue::Set(e.completed_at),
            duration_ms: ActiveValue::Set(e.duration_ms),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_delegation() -> Delegation {
        Delegation {
            id: "del-001".into(),
            parent_session_id: "ses-001".into(),
            child_session_id: "ses-002".into(),
            prompt: "Implement feature X".into(),
            prompt_embedding_id: Some("emb-001".into()),
            result: Some("Feature X implemented".into()),
            success: true,
            created_at: 1700000000,
            completed_at: Some(1700001000),
            duration_ms: Some(60000),
        }
    }

    #[test]
    fn round_trip_delegation() {
        let domain = sample_delegation();
        let active: delegation::ActiveModel = domain.clone().into();

        let model = delegation::Model {
            id: active.id.unwrap(),
            parent_session_id: active.parent_session_id.unwrap(),
            child_session_id: active.child_session_id.unwrap(),
            prompt: active.prompt.unwrap(),
            prompt_embedding_id: active.prompt_embedding_id.unwrap(),
            result: active.result.unwrap(),
            success: active.success.unwrap(),
            created_at: active.created_at.unwrap(),
            completed_at: active.completed_at.unwrap(),
            duration_ms: active.duration_ms.unwrap(),
        };

        let back: Delegation = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.success, true);
        assert_eq!(back.duration_ms, Some(60000));
    }

    #[test]
    fn round_trip_delegation_failed() {
        let mut domain = sample_delegation();
        domain.success = false;
        domain.result = None;
        domain.completed_at = None;
        domain.duration_ms = None;

        let active: delegation::ActiveModel = domain.clone().into();
        let model = delegation::Model {
            id: active.id.unwrap(),
            parent_session_id: active.parent_session_id.unwrap(),
            child_session_id: active.child_session_id.unwrap(),
            prompt: active.prompt.unwrap(),
            prompt_embedding_id: active.prompt_embedding_id.unwrap(),
            result: active.result.unwrap(),
            success: active.success.unwrap(),
            created_at: active.created_at.unwrap(),
            completed_at: active.completed_at.unwrap(),
            duration_ms: active.duration_ms.unwrap(),
        };

        let back: Delegation = model.into();
        assert_eq!(back.success, false);
        assert_eq!(back.result, None);
    }
}
