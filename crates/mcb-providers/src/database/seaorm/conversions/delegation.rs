//! Delegation entity ↔ `SeaORM` model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::delegation;
use mcb_domain::entities::Delegation;

crate::impl_conversion!(delegation, Delegation,
    direct: [id, parent_session_id, child_session_id, prompt, prompt_embedding_id,
             result, created_at, completed_at, duration_ms],
    bools: [success]
);
