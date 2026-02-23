//! AgentSession domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::agent_session;
use mcb_domain::entities::AgentSession;
use mcb_domain::entities::agent::{AgentSessionStatus, AgentType};

impl From<agent_session::Model> for AgentSession {
    fn from(m: agent_session::Model) -> Self {
        Self {
            id: m.id,
            session_summary_id: m.session_summary_id,
            agent_type: m
                .agent_type
                .parse::<AgentType>()
                .unwrap_or(AgentType::Sisyphus),
            model: m.model,
            parent_session_id: m.parent_session_id,
            started_at: m.started_at,
            ended_at: m.ended_at,
            duration_ms: m.duration_ms,
            status: m
                .status
                .parse::<AgentSessionStatus>()
                .unwrap_or(AgentSessionStatus::Active),
            prompt_summary: m.prompt_summary,
            result_summary: m.result_summary,
            token_count: m.token_count,
            tool_calls_count: m.tool_calls_count,
            delegations_count: m.delegations_count,
            project_id: m.project_id,
            worktree_id: m.worktree_id,
        }
    }
}

impl From<AgentSession> for agent_session::ActiveModel {
    fn from(e: AgentSession) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            project_id: ActiveValue::Set(e.project_id),
            worktree_id: ActiveValue::Set(e.worktree_id),
            session_summary_id: ActiveValue::Set(e.session_summary_id),
            agent_type: ActiveValue::Set(e.agent_type.to_string()),
            model: ActiveValue::Set(e.model),
            parent_session_id: ActiveValue::Set(e.parent_session_id),
            started_at: ActiveValue::Set(e.started_at),
            ended_at: ActiveValue::Set(e.ended_at),
            duration_ms: ActiveValue::Set(e.duration_ms),
            status: ActiveValue::Set(e.status.to_string()),
            prompt_summary: ActiveValue::Set(e.prompt_summary),
            result_summary: ActiveValue::Set(e.result_summary),
            token_count: ActiveValue::Set(e.token_count),
            tool_calls_count: ActiveValue::Set(e.tool_calls_count),
            delegations_count: ActiveValue::Set(e.delegations_count),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_agent_session() -> AgentSession {
        AgentSession {
            id: "ses-001".into(),
            session_summary_id: "ss-001".into(),
            agent_type: AgentType::Sisyphus,
            model: "claude-3-opus".into(),
            parent_session_id: None,
            started_at: 1700000000,
            ended_at: Some(1700001000),
            duration_ms: Some(1000000),
            status: AgentSessionStatus::Completed,
            prompt_summary: Some("Build feature X".into()),
            result_summary: Some("Feature X built".into()),
            token_count: Some(5000),
            tool_calls_count: Some(42),
            delegations_count: Some(3),
            project_id: Some("proj-001".into()),
            worktree_id: Some("wt-001".into()),
        }
    }

    #[test]
    fn round_trip_agent_session() {
        let domain = sample_agent_session();
        let active: agent_session::ActiveModel = domain.clone().into();

        let model = agent_session::Model {
            id: active.id.unwrap(),
            project_id: active.project_id.unwrap(),
            worktree_id: active.worktree_id.unwrap(),
            session_summary_id: active.session_summary_id.unwrap(),
            agent_type: active.agent_type.unwrap(),
            model: active.model.unwrap(),
            parent_session_id: active.parent_session_id.unwrap(),
            started_at: active.started_at.unwrap(),
            ended_at: active.ended_at.unwrap(),
            duration_ms: active.duration_ms.unwrap(),
            status: active.status.unwrap(),
            prompt_summary: active.prompt_summary.unwrap(),
            result_summary: active.result_summary.unwrap(),
            token_count: active.token_count.unwrap(),
            tool_calls_count: active.tool_calls_count.unwrap(),
            delegations_count: active.delegations_count.unwrap(),
        };

        let back: AgentSession = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.agent_type, domain.agent_type);
        assert_eq!(back.status, domain.status);
        assert_eq!(back.token_count, domain.token_count);
    }

    #[test]
    fn round_trip_agent_session_minimal() {
        let mut domain = sample_agent_session();
        domain.parent_session_id = None;
        domain.ended_at = None;
        domain.duration_ms = None;
        domain.prompt_summary = None;
        domain.result_summary = None;
        domain.token_count = None;
        domain.tool_calls_count = None;
        domain.delegations_count = None;
        domain.project_id = None;
        domain.worktree_id = None;
        domain.status = AgentSessionStatus::Active;

        let active: agent_session::ActiveModel = domain.clone().into();
        let model = agent_session::Model {
            id: active.id.unwrap(),
            project_id: active.project_id.unwrap(),
            worktree_id: active.worktree_id.unwrap(),
            session_summary_id: active.session_summary_id.unwrap(),
            agent_type: active.agent_type.unwrap(),
            model: active.model.unwrap(),
            parent_session_id: active.parent_session_id.unwrap(),
            started_at: active.started_at.unwrap(),
            ended_at: active.ended_at.unwrap(),
            duration_ms: active.duration_ms.unwrap(),
            status: active.status.unwrap(),
            prompt_summary: active.prompt_summary.unwrap(),
            result_summary: active.result_summary.unwrap(),
            token_count: active.token_count.unwrap(),
            tool_calls_count: active.tool_calls_count.unwrap(),
            delegations_count: active.delegations_count.unwrap(),
        };

        let back: AgentSession = model.into();
        assert_eq!(back.parent_session_id, None);
        assert_eq!(back.ended_at, None);
        assert_eq!(back.status, AgentSessionStatus::Active);
    }
}
