//! Observation domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::observation;
use mcb_domain::entities::observation::{Observation, ObservationMetadata, ObservationType};

impl From<observation::Model> for Observation {
    fn from(m: observation::Model) -> Self {
        let tags: Vec<String> = m
            .tags
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        let observation_type = m
            .observation_type
            .as_deref()
            .and_then(|s| s.parse::<ObservationType>().ok())
            .unwrap_or(ObservationType::Context);

        let metadata: ObservationMetadata = m
            .metadata
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or_default();

        Self {
            id: m.id,
            project_id: m.project_id,
            content: m.content,
            content_hash: m.content_hash,
            tags,
            r#type: observation_type,
            metadata,
            created_at: m.created_at,
            embedding_id: m.embedding_id,
        }
    }
}

impl From<Observation> for observation::ActiveModel {
    fn from(e: Observation) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            project_id: ActiveValue::Set(e.project_id),
            content: ActiveValue::Set(e.content),
            content_hash: ActiveValue::Set(e.content_hash),
            tags: ActiveValue::Set(Some(
                serde_json::to_string(&e.tags).unwrap_or_else(|_| "[]".into()),
            )),
            observation_type: ActiveValue::Set(Some(e.r#type.to_string())),
            metadata: ActiveValue::Set(Some(
                serde_json::to_string(&e.metadata).unwrap_or_else(|_| "{}".into()),
            )),
            created_at: ActiveValue::Set(e.created_at),
            embedding_id: ActiveValue::Set(e.embedding_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_observation() -> Observation {
        Observation {
            id: "obs-001".into(),
            project_id: "proj-001".into(),
            content: "Found a pattern in auth module".into(),
            content_hash: "sha256:abc".into(),
            tags: vec!["auth".into(), "pattern".into()],
            r#type: ObservationType::Code,
            metadata: ObservationMetadata::default(),
            created_at: 1700000000,
            embedding_id: Some("emb-001".into()),
        }
    }

    #[test]
    fn round_trip_observation() {
        let domain = sample_observation();
        let active: observation::ActiveModel = domain.clone().into();

        let model = observation::Model {
            id: active.id.unwrap(),
            project_id: active.project_id.unwrap(),
            content: active.content.unwrap(),
            content_hash: active.content_hash.unwrap(),
            tags: active.tags.unwrap(),
            observation_type: active.observation_type.unwrap(),
            metadata: active.metadata.unwrap(),
            created_at: active.created_at.unwrap(),
            embedding_id: active.embedding_id.unwrap(),
        };

        let back: Observation = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.tags, domain.tags);
        assert_eq!(back.r#type, ObservationType::Code);
        assert_eq!(back.embedding_id, domain.embedding_id);
    }
}
