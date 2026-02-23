//! TeamMember domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::team_member;
use mcb_domain::entities::team::{TeamMember, TeamMemberRole};
use mcb_domain::value_objects::ids::TeamMemberId;

impl From<team_member::Model> for TeamMember {
    fn from(m: team_member::Model) -> Self {
        Self {
            id: TeamMemberId::from(format!("{}:{}", m.team_id, m.user_id).as_str()),
            team_id: m.team_id,
            user_id: m.user_id,
            role: m
                .role
                .parse::<TeamMemberRole>()
                .unwrap_or(TeamMemberRole::Member),
            joined_at: m.joined_at,
        }
    }
}

impl From<TeamMember> for team_member::ActiveModel {
    fn from(e: TeamMember) -> Self {
        Self {
            team_id: ActiveValue::Set(e.team_id),
            user_id: ActiveValue::Set(e.user_id),
            role: ActiveValue::Set(e.role.to_string()),
            joined_at: ActiveValue::Set(e.joined_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_team_member() -> TeamMember {
        TeamMember {
            id: TeamMemberId::from("team-001:usr-001"),
            team_id: "team-001".into(),
            user_id: "usr-001".into(),
            role: TeamMemberRole::Lead,
            joined_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_team_member() {
        let domain = sample_team_member();
        let active: team_member::ActiveModel = domain.clone().into();

        let model = team_member::Model {
            team_id: active.team_id.unwrap(),
            user_id: active.user_id.unwrap(),
            role: active.role.unwrap(),
            joined_at: active.joined_at.unwrap(),
        };

        let back: TeamMember = model.into();
        assert_eq!(back.team_id, domain.team_id);
        assert_eq!(back.user_id, domain.user_id);
        assert_eq!(back.role, domain.role);
        assert_eq!(back.joined_at, domain.joined_at);
    }
}
