//! Worktree entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::worktree;
use mcb_domain::entities::Worktree;
use mcb_domain::entities::worktree::WorktreeStatus;

crate::impl_conversion!(worktree, Worktree,
    direct: [id, repository_id, branch_id, path, assigned_agent_id, created_at, updated_at],
    enums: { status: WorktreeStatus = WorktreeStatus::Active },
    not_set: [org_id, project_id, origin_context]
);
