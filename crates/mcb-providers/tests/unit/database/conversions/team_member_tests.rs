//! Tests for `team_member` conversion.

use mcb_domain::entities::TeamMember;
use mcb_providers::database::seaorm::entities::team_member;

fn sample_team_member() -> team_member::Model {
    team_member::Model {
        team_id: "ref_team_id_001".into(),
        user_id: "ref_user_id_001".into(),
        role: "Member".into(),
        joined_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_team_member() {
    let model = sample_team_member();
    let model_val = model.team_id.clone();

    // Model → Domain
    let domain: TeamMember = model.into();
    assert_eq!(domain.team_id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: team_member::ActiveModel = domain.into();
}
