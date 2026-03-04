//! Tool call entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::tool_call;
use mcb_domain::entities::ToolCall;

crate::impl_conversion!(tool_call, ToolCall,
    direct: [id, session_id, tool_name, params_summary, error_message, duration_ms, created_at],
    bools: [success],
    not_set: [org_id, project_id, repo_id]
);
