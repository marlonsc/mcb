//! Team domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::team;
use mcb_domain::entities::Team;

impl From<team::Model> for Team {
    fn from(m: team::Model) -> Self {
        Self {
            id: m.id,
            org_id: m.org_id,
            name: m.name,
            created_at: m.created_at,
        }
    }
}

impl From<Team> for team::ActiveModel {
    fn from(e: Team) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            org_id: ActiveValue::Set(e.org_id),
            name: ActiveValue::Set(e.name),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_team() -> Team {
        Team {
            id: "team-001".into(),
            org_id: "org-001".into(),
            name: "Backend Team".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_team() {
        let domain = sample_team();
        let active: team::ActiveModel = domain.clone().into();

        let model = team::Model {
            id: active.id.unwrap(),
            org_id: active.org_id.unwrap(),
            name: active.name.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: Team = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.name, domain.name);
    }
}
