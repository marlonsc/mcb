//! IssueComment domain ↔ SeaORM conversions.

use sea_orm::ActiveValue;

use crate::database::seaorm::entities::issue_comment;
use mcb_domain::entities::IssueComment;

impl From<issue_comment::Model> for IssueComment {
    fn from(m: issue_comment::Model) -> Self {
        Self {
            id: m.id,
            issue_id: m.issue_id,
            author_id: m.author_id,
            content: m.content,
            created_at: m.created_at,
        }
    }
}

impl From<IssueComment> for issue_comment::ActiveModel {
    fn from(e: IssueComment) -> Self {
        Self {
            id: ActiveValue::Set(e.id),
            issue_id: ActiveValue::Set(e.issue_id),
            author_id: ActiveValue::Set(e.author_id),
            content: ActiveValue::Set(e.content),
            created_at: ActiveValue::Set(e.created_at),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_issue_comment() -> IssueComment {
        IssueComment {
            id: "cmt-001".into(),
            issue_id: "iss-001".into(),
            author_id: "usr-001".into(),
            content: "This looks like a race condition".into(),
            created_at: 1700000000,
        }
    }

    #[test]
    fn round_trip_issue_comment() {
        let domain = sample_issue_comment();
        let active: issue_comment::ActiveModel = domain.clone().into();

        let model = issue_comment::Model {
            id: active.id.unwrap(),
            issue_id: active.issue_id.unwrap(),
            author_id: active.author_id.unwrap(),
            content: active.content.unwrap(),
            created_at: active.created_at.unwrap(),
        };

        let back: IssueComment = model.into();
        assert_eq!(back.id, domain.id);
        assert_eq!(back.content, domain.content);
    }
}
