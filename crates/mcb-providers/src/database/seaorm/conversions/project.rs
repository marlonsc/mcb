//! Project domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::project;
use mcb_domain::entities::Project;

impl From<project::Model> for Project {
    fn from(m: project::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            name: m.name,
            path: m.path,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

impl From<Project> for project::ActiveModel {
    fn from(e: Project) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            name: ActiveValue::Set(e.name),
            path: ActiveValue::Set(e.path),
            created_at: ActiveValue::Set(e.created_at),
            updated_at: ActiveValue::Set(e.updated_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_project() -> Project {
        Project {
            id: "proj-001".into(),
            org_id: "org-001".into(),
            name: "MCB".into(),
            path: "/home/user/mcb".into(),
            created_at: 1700000000,
            updated_at: 1700000001,
        }
    }

    #[test]
    fn round_trip_project() {
        let domain = sample_project();
        let active: project::ActiveModel = domain.clone().into();

        let model = project::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            name: active.name.unwrap(),
            path: active.path.unwrap(),
            created_at: active.created_at.unwrap(),
            updated_at: active.updated_at.unwrap(),
        };

        let back: Project = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.name, domain.name);
        assert_eq!(back.path, domain.path);
    }
}
