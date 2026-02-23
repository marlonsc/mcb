//! Repository domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::repository;
use mcb_domain::entities::Repository;
use mcb_domain::entities::repository::VcsType;

impl From<repository::Model> for Repository {
    fn from(m: repository::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            project_id: m.project_id,
            name: m.name,
            url: m.url,
            local_path: m.local_path,
            vcs_type: m.vcs_type.parse::<VcsType>().unwrap_or(VcsType::Git),
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Repository> for repository::ActiveModel {
    fn from(e: Repository) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            project_id: ActiveValue::Set(e.project_id),
            name: ActiveValue::Set(e.name),
            url: ActiveValue::Set(e.url),
            local_path: ActiveValue::Set(e.local_path),
            vcs_type: ActiveValue::Set(e.vcs_type.to_string()),
            origin_context: ActiveValue::NotSet,
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_repo() -> Repository {
        Repository {
            id: "repo-001".into(),
            org_id: "org-001".into(),
            project_id: "proj-001".into(),
            name: "mcb".into(),
            url: "https://github.com/user/mcb.git".into(),
            local_path: "/home/user/mcb".into(),
            vcs_type: VcsType::Git,
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_repository() {
        let domain = sample_repo();
        let active: repository::ActiveModel = domain.clone().into();

        let model = repository::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            project_id: active.project_id.unwrap(),
            name: active.name.unwrap(),
            url: active.url.unwrap(),
            local_path: active.local_path.unwrap(),
            vcs_type: active.vcs_type.unwrap(),
            origin_context: None,
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: Repository = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.vcs_type, domain.vcs_type);
        assert_eq!(back.url, domain.url);
    }
}
