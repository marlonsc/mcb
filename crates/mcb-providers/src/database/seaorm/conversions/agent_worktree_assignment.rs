//! Agent worktree assignment entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::agent_worktree_assignment;
use mcb_domain::entities::AgentWorktreeAssignment;

crate::impl_conversion!(agent_worktree_assignment, AgentWorktreeAssignment,
    direct: [id, agent_session_id, worktree_id, assigned_at, released_at],
    not_set: [origin_context]
);
