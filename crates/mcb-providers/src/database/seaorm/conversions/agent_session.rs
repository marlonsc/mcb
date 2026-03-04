//! Agent session entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::agent_session;
use mcb_domain::entities::AgentSession;
use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};

crate::impl_conversion!(agent_session, AgentSession,
    direct: [id, session_summary_id, model, parent_session_id, started_at, ended_at,
             duration_ms, prompt_summary, result_summary, token_count, tool_calls_count,
             delegations_count, project_id, worktree_id],
    enums: {
        agent_type: AgentType = AgentType::Sisyphus,
        status: AgentSessionStatus = AgentSessionStatus::Active,
    }
);
