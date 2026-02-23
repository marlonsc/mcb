//! AgentWorktreeAssignment domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::agent_worktree_assignment;
use mcb_domain::entities::AgentWorktreeAssignment;

impl From<agent_worktree_assignment::Model> for AgentWorktreeAssignment {
    fn from(m: agent_worktree_assignment::Model) -> Self {
        Self {
            id: m.id,
            agent_session_id: m.agent_session_id,
            worktree_id: m.worktree_id,
            assigned_at: m.assigned_at,
            released_at: m.released_at,
        }
    }
}

impl From<AgentWorktreeAssignment> for agent_worktree_assignment::ActiveModel {
    fn from(e: AgentWorktreeAssignment) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            agent_session_id: ActiveValue::Set(e.agent_session_id),
            worktree_id: ActiveValue::Set(e.worktree_id),
            assigned_at: ActiveValue::Set(e.assigned_at),
            released_at: ActiveValue::Set(e.released_at),
            origin_context: ActiveValue::NotSet,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_assignment() -> AgentWorktreeAssignment {
        AgentWorktreeAssignment {
            id: "asgn-001".into(),
            agent_session_id: "ses-001".into(),
            worktree_id: "wt-001".into(),
            assigned_at: 1700000000,
            released_at: Some(1700001000),
        }
    }

    #[test]
    fn round_trip_agent_worktree_assignment() {
        let domain = sample_assignment();
        let active: agent_worktree_assignment::ActiveModel = domain.clone().into();

        let model = agent_worktree_assignment::Model {
            id: active.id.unwrap(),
            agent_session_id: active.agent_session_id.unwrap(),
            worktree_id: active.worktree_id.unwrap(),
            assigned_at: active.assigned_at.unwrap(),
            released_at: active.released_at.unwrap(),
            origin_context: None,
        };

        let back: AgentWorktreeAssignment = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.released_at, Some(1700001000));
    }

    #[test]
    fn round_trip_active_assignment() {
        let mut domain = sample_assignment();
        domain.released_at = None;

        let active: agent_worktree_assignment::ActiveModel = domain.clone().into();
        let model = agent_worktree_assignment::Model {
            id: active.id.unwrap(),
            agent_session_id: active.agent_session_id.unwrap(),
            worktree_id: active.worktree_id.unwrap(),
            assigned_at: active.assigned_at.unwrap(),
            released_at: active.released_at.unwrap(),
            origin_context: None,
        };

        let back: AgentWorktreeAssignment = model.into();
        assert_eq!(back.released_at, None);
    }
}
