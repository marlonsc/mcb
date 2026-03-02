//! Tests for `team` conversion.

use mcb_domain::entities::Team;
use mcb_providers::database::seaorm::entities::team;
use rstest::rstest;

fn sample_team() -> team::Model {
    team::Model {
        id: "team_test_001".into(),
        org_id: "ref_org_id_001".into(),
        name: "test_name".into(),
        created_at: 1_700_000_000,
    }
}

#[rstest]
#[test]
fn round_trip_team() {
    let model = sample_team();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: Team = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: team::ActiveModel = domain.into();
}
