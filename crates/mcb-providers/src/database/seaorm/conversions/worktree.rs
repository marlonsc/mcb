//! Worktree domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::worktree;
use mcb_domain::entities::Worktree;
use mcb_domain::entities::worktree::WorktreeStatus;

impl From<worktree::Model> for Worktree {
    fn from(m: worktree::Model) -> Self {
        Self {
            id: m.id,
            repository_id: m.repository_id,
            branch_id: m.branch_id,
            path: m.path,
            status: m
                .status
                .parse::<WorktreeStatus>()
                .unwrap_or(WorktreeStatus::Active),
            assigned_agent_id: m.assigned_agent_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Worktree> for worktree::ActiveModel {
    fn from(e: Worktree) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::NotSet,
            project_id: ActiveValue::NotSet,
            repository_id: ActiveValue::Set(e.repository_id),
            branch_id: ActiveValue::Set(e.branch_id),
            path: ActiveValue::Set(e.path),
            status: ActiveValue::Set(e.status.to_string()),
            assigned_agent_id: ActiveValue::Set(e.assigned_agent_id),
            origin_context: ActiveValue::NotSet,
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_worktree() -> Worktree {
        Worktree {
            id: "wt-001".into(),
            repository_id: "repo-001".into(),
            branch_id: "br-001".into(),
            path: "/tmp/worktree".into(),
            status: WorktreeStatus::Active,
            assigned_agent_id: Some("agent-001".into()),
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_worktree() {
        let domain = sample_worktree();
        let active: worktree::ActiveModel = domain.clone().into();

        let model = worktree::Model {
            id: active.id.unwrap(),
            org_id: None,
            project_id: None,
            repository_id: active.repository_id.unwrap(),
            branch_id: active.branch_id.unwrap(),
            path: active.path.unwrap(),
            status: active.status.unwrap(),
            assigned_agent_id: active.assigned_agent_id.unwrap(),
            origin_context: None,
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: Worktree = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.status, domain.status);
        assert_eq!(back.assigned_agent_id, domain.assigned_agent_id);
    }
}
