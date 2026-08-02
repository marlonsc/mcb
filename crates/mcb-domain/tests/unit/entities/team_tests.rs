use mcb_domain::entities::team::{Team, TeamMember, TeamMemberRole};
use mcb_domain::value_objects::ids::TeamMemberId;
use mcb_utils::utils::id;
use rstest::{fixture, rstest};

#[fixture]
fn team() -> Team {
    Team {
        id: "team-001".to_owned(),
        org_id: "org-001".to_owned(),
        name: "Platform".to_owned(),
        created_at: 1000,
    }
}

#[fixture]
fn team_member(team: Team) -> TeamMember {
    let id_uuid = id::deterministic("team_member", format!("{}:usr-001", team.id).as_str());
    TeamMember {
        id: TeamMemberId::from_uuid(id_uuid),
        team_id: team.id,
        user_id: "usr-001".to_owned(),
        role: TeamMemberRole::Lead,
        joined_at: 1000,
    }
}

#[rstest]
fn test_team_construction(team: Team) {
    assert_eq!(team.id, "team-001");
    assert_eq!(team.name, "Platform");
}

#[rstest]
fn test_team_serialization_roundtrip(team: Team) {
    let json = serde_json::to_string(&team).expect("serialize");
    let deserialized: Team = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, team.id);
}

#[rstest]
fn test_team_member_construction(team_member: TeamMember) {
    assert_eq!(team_member.role, TeamMemberRole::Lead);
}

#[rstest]
fn test_team_member_serialization_roundtrip(team_member: TeamMember) {
    let json = serde_json::to_string(&team_member).expect("serialize");
    let deserialized: TeamMember = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, team_member.id);
}

#[rstest]
#[case(TeamMemberRole::Lead, "lead")]
#[case(TeamMemberRole::Member, "member")]
fn test_team_member_role_as_str(#[case] role: TeamMemberRole, #[case] expected: &str) {
    assert_eq!(role.as_str(), expected);
}

#[rstest]
#[case("lead", Ok(TeamMemberRole::Lead))]
#[case("member", Ok(TeamMemberRole::Member))]
#[case("LEAD", Ok(TeamMemberRole::Lead))]
#[case("Member", Ok(TeamMemberRole::Member))]
#[case("invalid", Err(()))]
fn test_team_member_role_from_str(
    #[case] input: &str,
    #[case] expected: Result<TeamMemberRole, ()>,
) {
    match expected {
        Ok(role) => assert_eq!(input.parse::<TeamMemberRole>(), Ok(role)),
        Err(()) => assert!(input.parse::<TeamMemberRole>().is_err()),
    }
}
