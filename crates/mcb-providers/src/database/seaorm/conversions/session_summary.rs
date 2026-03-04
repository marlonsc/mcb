//! Session summary entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::session_summary;
use mcb_domain::entities::memory::{OriginContext, SessionSummary};

crate::impl_conversion!(session_summary, SessionSummary,
    direct: [id, project_id, session_id, created_at],
    json_arrays: [topics, decisions, next_steps, key_files],
    json_objects_opt: { origin_context: OriginContext },
    unwrap_defaults: [org_id],
    not_set: [repo_id]
);
