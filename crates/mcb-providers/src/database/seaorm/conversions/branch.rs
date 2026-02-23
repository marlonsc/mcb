//! Branch domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::branch;
use mcb_domain::entities::Branch;

impl From<branch::Model> for Branch {
    fn from(m: branch::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            repository_id: m.repository_id,
            name: m.name,
            is_default: m.is_default != 0,
            head_commit: m.head_commit,
            upstream: m.upstream,
            created_at: m.created_at,
        }
    }
}

impl From<Branch> for branch::ActiveModel {
    fn from(e: Branch) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            project_id: ActiveValue::NotSet,
            repository_id: ActiveValue::Set(e.repository_id),
            name: ActiveValue::Set(e.name),
            is_default: ActiveValue::Set(i64::from(e.is_default)),
            head_commit: ActiveValue::Set(e.head_commit),
            upstream: ActiveValue::Set(e.upstream),
            origin_context: ActiveValue::NotSet,
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_branch() -> Branch {
        Branch {
            id: "br-001".into(),
            org_id: "org-001".into(),
            repository_id: "repo-001".into(),
            name: "main".into(),
            is_default: true,
            head_commit: "abc123def456".into(),
            upstream: Some("origin/main".into()),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_branch() {
        let domain = sample_branch();
        let active: branch::ActiveModel = domain.clone().into();

        let model = branch::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: None,
            repository_id: active.repository_id.unwrap(),
            name: active.name.unwrap(),
            is_default: active.is_default.unwrap(),
            head_commit: active.head_commit.unwrap(),
            upstream: active.upstream.unwrap(),
            origin_context: None,
            created_at: active.created_at.unwrap(),
        };

        let back: Branch = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.is_default, true);
        assert_eq!(back.upstream, Some("origin/main".into()));
    }

    #[test]
    fn round_trip_branch_no_upstream() {
        let mut domain = sample_branch();
        domain.is_default = false;
        domain.upstream = None;
        let active: branch::ActiveModel = domain.clone().into();

        let model = branch::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: None,
            repository_id: active.repository_id.unwrap(),
            name: active.name.unwrap(),
            is_default: active.is_default.unwrap(),
            head_commit: active.head_commit.unwrap(),
            upstream: active.upstream.unwrap(),
            origin_context: None,
            created_at: active.created_at.unwrap(),
        };

        let back: Branch = model.into();
        assert_eq!(back.is_default, false);
        assert_eq!(back.upstream, None);
    }
}
