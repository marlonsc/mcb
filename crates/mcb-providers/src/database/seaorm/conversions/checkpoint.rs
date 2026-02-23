//! Checkpoint domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::checkpoint;
use mcb_domain::entities::Checkpoint;
use mcb_domain::entities::agent::CheckpointType;

impl From<checkpoint::Model> for Checkpoint {
    fn from(m: checkpoint::Model) -> Self {
        let snapshot_data: serde_json::Value =
            serde_json::from_str(&m.snapshot_data).unwrap_or(serde_json::Value::Null);

        Self {
            id: m.id,
            session_id: m.session_id,
            checkpoint_type: m
                .checkpoint_type
                .parse::<CheckpointType>()
                .unwrap_or(CheckpointType::File),
            description: m.description,
            snapshot_data,
            created_at: m.created_at,
            restored_at: m.restored_at,
            expired: m.expired.map_or(false, |v| v != 0),
        }
    }
}

impl From<Checkpoint> for checkpoint::ActiveModel {
    fn from(e: Checkpoint) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            session_id: ActiveValue::Set(e.session_id),
            checkpoint_type: ActiveValue::Set(e.checkpoint_type.to_string()),
            description: ActiveValue::Set(e.description),
            snapshot_data: ActiveValue::Set(
                serde_json::to_string(&e.snapshot_data).unwrap_or_default(),
            ),
            created_at: ActiveValue::Set(e.created_at),
            restored_at: ActiveValue::Set(e.restored_at),
            expired: ActiveValue::Set(Some(i64::from(e.expired))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_checkpoint() -> Checkpoint {
        Checkpoint {
            id: "cp-001".into(),
            session_id: "ses-001".into(),
            checkpoint_type: CheckpointType::Git,
            description: "Pre-refactor checkpoint".into(),
            snapshot_data: serde_json::json!({"branch": "main", "commit": "abc123"}),
            created_at: 1700000000,
            restored_at: None,
            expired: false,
        }
    }

    #[test]
    fn round_trip_checkpoint() {
        let domain = sample_checkpoint();
        let active: checkpoint::ActiveModel = domain.clone().into();

        let model = checkpoint::Model {
            id: active.id.unwrap(),
            session_id: active.session_id.unwrap(),
            checkpoint_type: active.checkpoint_type.unwrap(),
            description: active.description.unwrap(),
            snapshot_data: active.snapshot_data.unwrap(),
            created_at: active.created_at.unwrap(),
            restored_at: active.restored_at.unwrap(),
            expired: active.expired.unwrap(),
        };

        let back: Checkpoint = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.checkpoint_type, domain.checkpoint_type);
        assert_eq!(back.expired, false);
        assert_eq!(back.snapshot_data["branch"], "main");
    }

    #[test]
    fn round_trip_checkpoint_expired() {
        let mut domain = sample_checkpoint();
        domain.expired = true;
        domain.restored_at = Some(1700002000);

        let active: checkpoint::ActiveModel = domain.clone().into();
        let model = checkpoint::Model {
            id: active.id.unwrap(),
            session_id: active.session_id.unwrap(),
            checkpoint_type: active.checkpoint_type.unwrap(),
            description: active.description.unwrap(),
            snapshot_data: active.snapshot_data.unwrap(),
            created_at: active.created_at.unwrap(),
            restored_at: active.restored_at.unwrap(),
            expired: active.expired.unwrap(),
        };

        let back: Checkpoint = model.into();
        assert_eq!(back.expired, true);
        assert_eq!(back.restored_at, Some(1700002000));
    }
}
