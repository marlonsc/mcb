//! Observation entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::observation;
use mcb_domain::entities::observation::{Observation, ObservationMetadata, ObservationType};

crate::impl_conversion!(observation, Observation,
    direct: [id, project_id, content, content_hash, created_at, embedding_id],
    enum_opts: { observation_type as r#type: ObservationType = ObservationType::Context },
    json_arrays: [tags],
    json_objects: { metadata: ObservationMetadata }
);
