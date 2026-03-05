//! Checkpoint entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::checkpoint;
use mcb_domain::entities::Checkpoint;
use mcb_domain::entities::agent::CheckpointType;

crate::impl_conversion!(checkpoint, Checkpoint,
    direct: [id, session_id, description, created_at, restored_at],
    enums: { checkpoint_type: CheckpointType = CheckpointType::File },
    bool_opts: [expired],
    json_values: [snapshot_data]
);
