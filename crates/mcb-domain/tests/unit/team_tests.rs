use mcb_domain::entities::team::{Team, TeamMember, TeamMemberRole};

#[test]
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

#[test]
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

#[test]
fn team_member_construction() {
    let member = TeamMember {
        team_id: "team-001".to_string(),
        user_id: "usr-001".to_string(),
        role: TeamMemberRole::Lead,
        joined_at: 1000,
    };
    assert_eq!(member.team_id, "team-001");
    assert_eq!(member.user_id, "usr-001");
    assert_eq!(member.role, TeamMemberRole::Lead);
}

#[test]
fn team_member_role_as_str() {
    assert_eq!(TeamMemberRole::Lead.as_str(), "lead");
    assert_eq!(TeamMemberRole::Member.as_str(), "member");
}

#[test]
fn team_member_role_from_str() {
    assert_eq!("lead".parse::<TeamMemberRole>(), Ok(TeamMemberRole::Lead));
    assert_eq!(
        "member".parse::<TeamMemberRole>(),
        Ok(TeamMemberRole::Member)
    );
    assert!("invalid".parse::<TeamMemberRole>().is_err());
}

#[test]
fn team_member_role_from_str_case_insensitive() {
    assert_eq!("LEAD".parse::<TeamMemberRole>(), Ok(TeamMemberRole::Lead));
    assert_eq!(
        "Member".parse::<TeamMemberRole>(),
        Ok(TeamMemberRole::Member)
    );
}
