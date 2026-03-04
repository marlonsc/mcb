//! Team member entity ↔ SeaORM model conversions via `impl_conversion!`.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::team_member;
use mcb_domain::entities::team::{TeamMember, TeamMemberRole};
use mcb_domain::value_objects::ids::TeamMemberId;

crate::impl_conversion!(team_member, TeamMember,
    direct: [team_id, user_id, joined_at],
    enums: { role: TeamMemberRole = TeamMemberRole::Member },
    computed: { |m| id = TeamMemberId::from(format!("{}:{}", m.team_id, m.user_id).as_str()) },
    not_set: [id],
);
