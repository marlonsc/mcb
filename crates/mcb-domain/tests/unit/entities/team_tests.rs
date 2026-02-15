use mcb_domain::entities::team::{Team, TeamMember, TeamMemberRole};
use rstest::rstest;

#[rstest]
fn team_construction() {
    let team = Team {
        id: "team-001".to_string(),
        org_id: "org-001".to_string(),
        name: "Platform".to_string(),
        created_at: 1000,
    };
    assert_eq!(team.id, "team-001");
    assert_eq!(team.org_id, "org-001");
    assert_eq!(team.name, "Platform");
}

#[rstest]
fn team_serialization_roundtrip() {
    let team = Team {
        id: "team-002".to_string(),
        org_id: "org-001".to_string(),
        name: "Backend".to_string(),
        created_at: 2000,
    };
    let json = serde_json::to_string(&team).expect("serialize");
    let deserialized: Team = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.id, "team-002");
    assert_eq!(deserialized.name, "Backend");
}

use mcb_domain::utils::id;
use mcb_domain::value_objects::ids::TeamMemberId;

#[rstest]
fn team_member_construction() {
    let id_uuid = id::deterministic("team_member", "team-001:usr-001");
    let member = TeamMember {
        id: TeamMemberId::from_uuid(id_uuid),
        team_id: "team-001".to_string(),
        user_id: "usr-001".to_string(),
        role: TeamMemberRole::Lead,
        joined_at: 1000,
    };
    assert_eq!(member.team_id, "team-001");
    assert_eq!(member.user_id, "usr-001");
    assert_eq!(member.role, TeamMemberRole::Lead);
}

#[rstest]
#[case(TeamMemberRole::Lead, "lead")]
#[case(TeamMemberRole::Member, "member")]
fn team_member_role_as_str(#[case] role: TeamMemberRole, #[case] expected: &str) {
    assert_eq!(role.as_str(), expected);
}

#[rstest]
#[case("lead", Ok(TeamMemberRole::Lead))]
#[case("member", Ok(TeamMemberRole::Member))]
#[case("LEAD", Ok(TeamMemberRole::Lead))]
#[case("Member", Ok(TeamMemberRole::Member))]
#[case("invalid", Err(()))]
fn team_member_role_from_str(#[case] input: &str, #[case] expected: Result<TeamMemberRole, ()>) {
    match expected {
        Ok(role) => assert_eq!(input.parse::<TeamMemberRole>(), Ok(role)),
        Err(()) => assert!(input.parse::<TeamMemberRole>().is_err()),
    }
}
