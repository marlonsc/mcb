//! Tests for `issue_comment` conversion.

use mcb_domain::entities::IssueComment;
use mcb_providers::database::seaorm::entities::issue_comment;

fn sample_issue_comment() -> issue_comment::Model {
    issue_comment::Model {
        id: "issue_comment_test_001".into(),
        issue_id: "ref_issue_id_001".into(),
        author_id: "ref_author_id_001".into(),
        content: "test_content".into(),
        created_at: 1_700_000_000,
    }
}

#[test]
fn round_trip_issue_comment() {
    let model = sample_issue_comment();
    let model_val = model.id.clone();

    // Model → Domain
    let domain: IssueComment = model.into();
    assert_eq!(domain.id, model_val);

    // Domain → ActiveModel (should not panic)
    let _active: issue_comment::ActiveModel = domain.into();
}
